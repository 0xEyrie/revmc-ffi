package state

import (
	revmtypes "github.com/0xEyrie/revmc-ffi/types"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	gethstate "github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/holiman/uint256"
	"google.golang.org/protobuf/proto"
)

// StateDB to support revm ffi call.
type StateDB struct {
	trie          gethstate.Trie
	cachedb       gethstate.CachingDB
	reader        gethstate.Reader
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

/******Getter******/

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

func (state *StateDB) SetBlockHashFunc(blockHashFunc func(number uint64) common.Hash) {
	state.blockHashFunc = blockHashFunc
}

/******Commit******/
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
	for addr, account := range accountsbuf.GetAccounts() {
		stateAccount := &types.StateAccount{
			Nonce:    account.Nonce,
			Balance:  uint256.NewInt(0).SetBytes(account.Balance),
			CodeHash: account.CodeHash,
			// TODO: set the hash?
			Root: common.Hash{},
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

/******Code******/
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

/******Storages******/
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

/******Accounts******/
func (state *StateDB) updateAccounts(accounts map[common.Address]*types.StateAccount) error {
	for addr, account := range accounts {
		code := state.GetCodeByHash(account.CodeHash)
		err := state.trie.UpdateAccount(addr, account, len(code))
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
