package state

import (
	revmtypes "github.com/0xEyrie/revmffi/core/types"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/vm"
	"google.golang.org/protobuf/proto"
)

// Interface for go code
// StateDBFFI is an interface that defines methods for interacting with the state database.
// This interface is intended to be implemented in Rust code
type StateDBFFI interface {
	Basic(addr []byte) []byte
	GetCodeByHash(ch []byte) []byte
	GetStorage(addr []byte, k []byte) []byte
	GetBlockHash(number uint64) []byte
	UpdateAndCommit(s []byte, acc []byte, del []byte) (common.Hash, error)
}

type BlockMetadata interface {
	SetBlockNumber(number uint64)
	SetBlockHashFn(blockHashFn vm.GetHashFunc)
}

// ExtendedStateDB
type ExtendedStateDB struct {
	*state.StateDB
	BlockMetadata
	blockHashFunc func(number uint64) common.Hash
	blockNumber   uint64
}

var _ StateDBFFI = (*ExtendedStateDB)(nil)
var _ BlockMetadata = (*ExtendedStateDB)(nil)

// New creates a new state from a given trie.
func New(root common.Hash, db state.Database) (*ExtendedStateDB, error) {
	statedb, err := state.New(root, db)
	if err != nil {
		return nil, err
	}
	return &ExtendedStateDB{
		StateDB: statedb,
	}, nil
}

// ffi call from basic
func (state *ExtendedStateDB) Basic(addr []byte) []byte {
	address := common.BytesToAddress(addr)
	obj := state.GetStateObject(address)
	account, err := proto.Marshal(&revmtypes.Account{
		Balance:  obj.Balance().Bytes(),
		Nonce:    obj.Nonce(),
		CodeHash: obj.CodeHash(),
	})
	if err != nil {
		panic("failed to marshal proto message" + err.Error())
	}
	return account
}

// ffi call from code_by_hash
func (state *ExtendedStateDB) GetCodeByHash(ch []byte) []byte {
	codeHash := common.BytesToHash(ch)
	code, err := state.Reader.Code(common.Address{}, codeHash)
	if err != nil {
		panic("failed to marshal proto message" + err.Error())
	}
	return code
}

func (state *ExtendedStateDB) SetBlockHashFn(blockHashFn vm.GetHashFunc) {
	state.blockHashFunc = blockHashFn
}

func (state *ExtendedStateDB) SetBlockNumber(number uint64) {
	state.blockNumber = number
}

// ffi call from block_hash
func (state *ExtendedStateDB) GetBlockHash(number uint64) []byte {
	// GetBlockHash should be set on evm instance creation
	if state.blockHashFunc == nil {
		panic("blockHashFunc is not set")
	}
	return state.blockHashFunc(number).Bytes()
}

// ffi call from storage
func (state *ExtendedStateDB) GetStorage(addr []byte, k []byte) []byte {
	address := common.BytesToAddress(addr)
	hash := common.BytesToHash(k)
	obj := state.GetState(address, hash)
	return obj.Bytes()
}

func (state *ExtendedStateDB) UpdateAndCommit(s []byte, acc []byte, del []byte) (common.Hash, error) {
	// set storages
	var storagesbuf revmtypes.Storages
	err := proto.Unmarshal(s, &storagesbuf)
	if err != nil {
		return common.Hash{}, err
	}
	for addr, kv := range storagesbuf.GetStorages() {
		storage := make(map[common.Hash]common.Hash)
		for key, value := range kv.GetStorage() {
			storage[common.HexToHash(key)] = common.BytesToHash(value)
		}
		state.SetStorage(common.HexToAddress(addr), storage)
	}
	// set accounts
	var accountsbuf revmtypes.Accounts
	err = proto.Unmarshal(acc, &accountsbuf)
	if err != nil {
		return common.Hash{}, err
	}
	for addr, acc := range accountsbuf.GetAccounts() {
		code, stateacc := acc.Into()
		state.SetAccount(common.HexToAddress(addr), &stateacc)
		state.SetCode(common.HexToAddress(addr), code)
	}
	// delete contract by self destructed opcodes
	var deletedbuf revmtypes.Deleted
	err = proto.Unmarshal(del, &deletedbuf)
	if err != nil {
		return common.Hash{}, err
	}
	for _, del := range deletedbuf.GetDeleted() {
		state.SelfDestruct6780(common.BytesToAddress(del))
	}

	// commit updated codes
	root, err := state.Commit(state.blockNumber, true)
	if err != nil {
		return common.Hash{}, err
	}

	return root, nil
}
