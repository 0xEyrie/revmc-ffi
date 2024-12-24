package revm

import (
	revmtypes "github.com/0xEyrie/revmc-ffi/types"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	gethstate "github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/tracing"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/holiman/uint256"
	"google.golang.org/protobuf/proto"
)

// Interface for go code
// StateDBI is an interface that defines methods for interacting with the state database.
// This interface is intended to be implemented in Go and used in conjunction with Rust code.
type StateDBI interface {
	GetStateAccount(common.Address) *types.StateAccount

	GetBalance(common.Address) *uint256.Int
	SubBalance(common.Address, *uint256.Int, tracing.BalanceChangeReason) uint256.Int

	GetLogs(hash common.Hash, blockNumber uint64, blockHash common.Hash) []*types.Log
	AddLog(log *revmtypes.Log)
	SetTxContext(thash common.Hash, ti int)
}

type StateDBFFI interface {
	GetAccount(addr []byte) []byte
	GetCodeByHash(ch []byte) []byte
	GetStorage(addr []byte, k []byte) []byte
	GetBlockHash(number uint64) []byte
	Commit(c []byte, s []byte, acc []byte, del []byte) (common.Hash, error)

	SetGetBlockHash(blockHashFn vm.GetHashFunc)
}

// StateDB to support revm ffi call.
type StateDB struct {
	trie    gethstate.Trie
	cachedb gethstate.CachingDB
	reader  gethstate.Reader

	// The tx context and all occurred logs in the scope of transaction.
	thash   common.Hash
	txIndex int
	logs    map[common.Hash][]*types.Log
	logSize uint

	blockHashFunc func(number uint64) common.Hash
}

// New creates a new state from a given trie.
func New(root common.Hash, cachedb gethstate.CachingDB) (*StateDB, error) {
	reader, err := cachedb.Reader(root)
	if err != nil {
		return nil, err
	}

	trie, err := cachedb.OpenTrie(root)
	if err != nil {
		return nil, err
	}

	return &StateDB{
		trie:          trie,
		cachedb:       cachedb,
		reader:        reader,
		blockHashFunc: nil,
	}, nil
}

func (state *StateDB) GetBalance(addr common.Address) *uint256.Int {
	acc, err := state.reader.Account(addr)

	if err != nil {
		panic("failed to get account: " + err.Error())
	}
	return acc.Balance
}

// SubBalance subtracts amount from the account associated with addr.
func (state *StateDB) SubBalance(addr common.Address, amount *uint256.Int, reason tracing.BalanceChangeReason) uint256.Int {
	stacc := state.GetStateAccount(addr)
	if stacc == nil {
		return uint256.Int{}
	}
	prev := stacc.Balance
	if amount.IsZero() {
		return *prev
	}

	balance := new(uint256.Int).Sub(prev, amount)
	err := state.updateAccount(addr, &types.StateAccount{
		Nonce:    stacc.Nonce,
		Balance:  balance,
		Root:     stacc.Root,
		CodeHash: stacc.CodeHash,
	})
	if err != nil {
		return *prev
	}

	return *balance
}

func (state *StateDB) GetStateAccount(addr common.Address) *types.StateAccount {
	acc, err := state.reader.Account(addr)
	if err != nil {
		panic("failed to get account: " + err.Error())
	}
	return acc
}

// SetTxContext sets the current transaction hash and index which are
// used when the EVM emits new state logs. It should be invoked before
// transaction execution.
func (s *StateDB) SetTxContext(thash common.Hash, ti int) {
	s.thash = thash
	s.txIndex = ti
}

func (state *StateDB) AddLog(log *types.Log) {
	log.TxHash = state.thash
	log.TxIndex = uint(state.txIndex)
	log.Index = state.logSize
	state.logs[state.thash] = append(state.logs[state.thash], log)
	state.logSize++
}

// GetLogs returns the logs matching the specified transaction hash, and annotates
// them with the given blockNumber and blockHash.
func (s *StateDB) GetLogs(hash common.Hash, blockNumber uint64, blockHash common.Hash) []*types.Log {
	logs := s.logs[hash]
	for _, l := range logs {
		l.BlockNumber = blockNumber
		l.BlockHash = blockHash
	}
	return logs
}

func (state *StateDB) GetAccount(addr []byte) []byte {
	address := common.BytesToAddress(addr)
	acc, err := state.reader.Account(address)

	if err != nil {
		panic("failed to get account: " + err.Error())
	}
	account, err := proto.Marshal(&revmtypes.Account{
		Balance:  acc.Balance.Bytes(),
		Nonce:    acc.Nonce,
		CodeHash: acc.CodeHash,
	})
	if err != nil {
		panic("failed to marshal proto message" + err.Error())
	}
	return account
}

func (state *StateDB) GetCodeByHash(ch []byte) []byte {
	codeHash := common.BytesToHash(ch)
	code, err := state.reader.Code(common.Address{}, codeHash)
	if err != nil {
		panic("failed to get code: " + err.Error())
	}

	return code
}

func (state *StateDB) GetStorage(addr []byte, k []byte) []byte {
	address := common.BytesToAddress(addr)
	key := common.BytesToHash(k)
	storage, err := state.reader.Storage(address, key)
	if err != nil {
		panic("failed to get storage: " + err.Error())
	}

	return storage.Bytes()
}

func (state *StateDB) GetBlockHash(number uint64) []byte {
	// GetBlockHash should be set on evm instance creation
	if state.blockHashFunc == nil {
		panic("blockHashFunc is not set")
	}
	return state.blockHashFunc(number).Bytes()
}

func (state *StateDB) SetGetBlockHash(blockHashFn vm.GetHashFunc) {
	state.blockHashFunc = blockHashFn
}

func (state *StateDB) Commit(c []byte, s []byte, acc []byte, del []byte) (common.Hash, error) {
	var storagesbuf revmtypes.Storages
	err := proto.Unmarshal(s, &storagesbuf)
	if err != nil {
		return common.Hash{}, err
	}
	storages := make(map[common.Address]map[common.Hash]common.Hash)
	for addr, kv := range storagesbuf.GetStorages() {
		for key, value := range kv.GetStorage() {
			storage := make(map[common.Hash]common.Hash)
			storage[common.HexToHash(key)] = common.BytesToHash(value)
			storages[common.HexToAddress(addr)] = storage
		}
	}
	// storages update on cachedb
	err = state.updateStorages(storages)
	if err != nil {
		return common.Hash{}, err
	}

	var accountsbuf revmtypes.Accounts
	err = proto.Unmarshal(acc, &accountsbuf)
	if err != nil {
		return common.Hash{}, err
	}
	accounts := make(map[common.Address]*types.StateAccount)
	for addr, _ := range accountsbuf.GetAccounts() {
		stateAccount, err := state.reader.Account(common.HexToAddress(addr))
		if err != nil {
			return common.Hash{}, err
		}
		accounts[common.HexToAddress(addr)] = stateAccount
	}
	// accounts update on cachedb
	err = state.updateAccounts(accounts)
	if err != nil {
		return common.Hash{}, err
	}

	var deletedbuf revmtypes.Deleted
	err = proto.Unmarshal(del, &deletedbuf)
	if err != nil {
		return common.Hash{}, err
	}
	deleted := make([]common.Address, len(deletedbuf.GetDeleted()))
	for _, del := range deletedbuf.GetDeleted() {
		deleted = append(deleted, common.BytesToAddress(del))
	}

	err = state.deleteStorages(deleted)
	if err != nil {
		return common.Hash{}, err
	}

	err = state.deleteAccounts(deleted)
	if err != nil {
		return common.Hash{}, err
	}
	// commit updated codes
	root, _ := state.trie.Commit(true)

	var codesbuf revmtypes.Codes
	err = proto.Unmarshal(c, &codesbuf)
	if err != nil {
		return common.Hash{}, err
	}
	codes := make(map[common.Hash][]byte)
	for codeHash, code := range codesbuf.GetCodes() {
		if state.GetCodeByHash([]byte(codeHash)) != nil {
			continue
		}
		hash := common.HexToHash(codeHash)
		codes[hash] = code
	}
	err = state.commitUpdatedCode(codes)

	// update reader
	if err != nil {
		return common.Hash{}, err
	}
	state.reader, err = state.cachedb.Reader(root)
	if err != nil {
		return root, err
	}
	return root, nil
}

// There is no interface for update in cachedb. so directly commit on rawdb
func (state *StateDB) commitUpdatedCode(codes map[common.Hash][]byte) error {
	if db := state.cachedb.TrieDB().Disk(); db != nil {
		batch := db.NewBatch()
		for codeHash, code := range codes {
			rawdb.WriteCode(batch, codeHash, code)
		}
		if err := batch.Write(); err != nil {
			return err
		}
	}
	return nil
}

func (state *StateDB) updateStorages(storages map[common.Address]map[common.Hash]common.Hash) error {
	for addr, kv := range storages {
		for key, value := range kv {
			err := state.trie.UpdateStorage(addr, key.Bytes(), value.Bytes())
			if err != nil {
				return err
			}
		}
	}
	return nil
}

func (state *StateDB) deleteStorages(addrs []common.Address) error {
	for _, addr := range addrs {
		err := state.trie.DeleteAccount(addr)
		if err != nil {
			return err
		}
	}
	return nil
}

func (state *StateDB) updateAccount(addr common.Address, acc *types.StateAccount) error {
	code := state.GetCodeByHash(acc.CodeHash)
	err := state.trie.UpdateAccount(addr, acc, len(code))
	if err != nil {
		return err
	}
	return nil
}

func (state *StateDB) updateAccounts(accounts map[common.Address]*types.StateAccount) error {
	for addr, acc := range accounts {
		err := state.updateAccount(addr, acc)
		if err != nil {
			return err
		}
	}
	return nil
}

func (state *StateDB) deleteAccounts(addrs []common.Address) error {
	for _, addr := range addrs {
		err := state.trie.DeleteAccount(addr)
		if err != nil {
			return err
		}
	}
	return nil
}
