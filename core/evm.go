package core

import (
	"math/big"

	"github.com/0xEyrie/revmc-ffi/core/revm"
	"github.com/0xEyrie/revmc-ffi/types"
	"github.com/ethereum/go-ethereum/common"
	gethcore "github.com/ethereum/go-ethereum/core"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"google.golang.org/protobuf/proto"
)

type Config struct {
	spec revm.SpecId

	NoBaseFee bool
	// TODO: Tracer
	// Compiler Setting
	hasCompiler       bool
	thershold         uint64
	maxConcurrentSize uint
}

// VM struct is the core of initiavm.
type EVM struct {
	Inner   revm.EVM
	Context vm.BlockContext
	// virtual machine configuration options used to initialise the
	// evm.
	Config Config
	vm.TxContext
}

// NewVM return VM instance
func NewEVM(blockCtx vm.BlockContext, statedb revm.StateDB, config Config) EVM {
	var inner revm.EVM
	if config.hasCompiler {
		inner = revm.NewVMWithCompiler(statedb, config.thershold, config.maxConcurrentSize, config.spec)
	} else {
		inner = revm.NewVM(statedb, config.spec)
	}

	return EVM{
		Inner:   inner,
		Context: blockCtx,
		Config:  config,
	}
}

func (evm *EVM) GetSpecId() revm.SpecId {
	return evm.Config.spec
}

func (evm *EVM) SetTxContext(txCtx vm.TxContext) {
	evm.TxContext = txCtx
}

func (evm *EVM) Destroy() {
	revm.DestroyVM(evm.Inner)
}

func (evm *EVM) SetGetHashFn(hashFn vm.GetHashFunc) {
	evm.Inner.StateDB.SetGetBlockHash(hashFn)
}

// Call execute transaction based on revm
// this function only support entry call of transactions
func (evm *EVM) Execute(
	caller vm.ContractRef, msg *gethcore.Message,
) (*types.EvmResult, error) {
	// save block context on evm
	excessBlobGas := evm.Context.BlobBaseFee.Uint64()
	block := &types.Block{
		Number:        evm.Context.BlockNumber.Bytes(),
		Coinbase:      evm.Context.Coinbase.Bytes(),
		Timestamp:     big.NewInt(int64(evm.Context.Time)).Bytes(),
		GasLimit:      big.NewInt(int64(evm.Context.GasLimit)).Bytes(),
		Basefee:       evm.Context.BaseFee.Bytes(),
		Difficulty:    evm.Context.Difficulty.Bytes(),
		Prevrandao:    evm.Context.Random[:],
		ExcessBlobGas: &excessBlobGas,
	}

	blockBuf, err := proto.Marshal(block)
	if err != nil {
		return nil, err
	}
	transaction := types.Transaction{
		Caller:         caller.Address().Bytes(),
		GasLimit:       msg.GasLimit,
		GasPrice:       msg.GasPrice.Bytes(),
		Nonce:          nil,
		TransactTo:     msg.To.Bytes(),
		Value:          msg.Value.Bytes(),
		Data:           msg.Data,
		GasPriorityFee: msg.GasTipCap.Bytes(),
		AccessList: func(accl gethtypes.AccessList) []*types.AccessListItem {
			result := make([]*types.AccessListItem, len(accl))
			for i, acc := range accl {
				storageKeys := make([]*types.StorageKey, len(acc.StorageKeys))
				for j, key := range acc.StorageKeys {
					storageKeys[j] = &types.StorageKey{Value: key.Bytes()}
				}
				result[i] = &types.AccessListItem{
					Address:     acc.Address.Bytes(),
					StorageKeys: storageKeys,
				}
			}
			return result
		}(msg.AccessList),
		BlobHashes: func(hashes []common.Hash) [][]byte {
			result := make([][]byte, len(hashes))
			for i, hash := range hashes {
				result[i] = hash.Bytes()
			}
			return result
		}(msg.BlobHashes),
		MaxFeePerBlobGas:  msg.BlobGasFeeCap.Bytes(),
		AuthorizationList: nil,
	}

	txBuf, err := proto.Marshal(&transaction)
	if err != nil {
		return nil, err
	}

	res, err := evm.Inner.Execute(
		&blockBuf,
		&txBuf,
	)
	if err != nil {
		return nil, err
	}
	return res, nil
}

// // StaticCall execute transaction based on revm
// // this function only supports entry StaticCall of transactions
// func (evm *EVM) StaticCall(
// 	caller vm.ContractRef,
// 	addr common.Address, input []byte, gas uint64,
// ) (ret []byte, leftOverGas uint64, err error) {
// 	// save block context on evm
// 	excessBlobGas := evm.Context.BlobBaseFee.Uint64()
// 	blockProto := &types.Block{
// 		Number:        evm.Context.BlockNumber.Bytes(),
// 		Coinbase:      evm.Context.Coinbase.Bytes(),
// 		Timestamp:     big.NewInt(int64(evm.Context.Time)).Bytes(),
// 		GasLimit:      big.NewInt(int64(evm.Context.GasLimit)).Bytes(),
// 		Basefee:       evm.Context.BaseFee.Bytes(),
// 		Difficulty:    evm.Context.Difficulty.Bytes(),
// 		Prevrandao:    evm.Context.Random[:],
// 		ExcessBlobGas: &excessBlobGas,
// 	}

// 	// save tx context on evm
// 	res, err := evm.Inner.Simulate(
// 		blockProto,
// 		&tx,
// 	)
// 	if err != nil {
// 		return nil, gas, err
// 	}

// 	return res, nil
// }
