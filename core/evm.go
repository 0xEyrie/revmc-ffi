package core

import (
	"math/big"

	"github.com/0xEyrie/revmffi/core/state"
	revmtypes "github.com/0xEyrie/revmffi/core/types"
	revm "github.com/0xEyrie/revmffi/core/vm"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"google.golang.org/protobuf/proto"
)

type Config struct {
	spec revm.SpecId

	NoBaseFee         bool
	hasCompiler       bool
	thershold         uint64
	maxConcurrentSize uint
}

// VM struct is the core of initiavm.
type EVM struct {
	Inner   revm.EVM
	Context vm.BlockContext
	// virtual machine configuration options used to initialise the evm.
	Config Config
	vm.TxContext
}

// NewVM return VM instance
func NewEVM(blockCtx vm.BlockContext, statedb state.ExtendedStateDB, config Config) EVM {
	var inner revm.EVM
	if config.hasCompiler {
		inner = revm.NewEVMWithCompiler(statedb, config.thershold, config.maxConcurrentSize, config.spec)
	} else {
		inner = revm.NewEVM(statedb, config.spec)
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

func (evm *EVM) SetBlockHashFn(hashFn vm.GetHashFunc) {
	evm.Inner.StateDB.SetBlockHashFn(hashFn)
}

func (evm *EVM) SetBlockNumber(number uint64) {
	evm.Inner.StateDB.SetBlockNumber(number)
}

// Call execute transaction based on revm
// this function only support entry call of transactions
func (evm *EVM) Execute(
	caller vm.ContractRef, msg *core.Message,
) (*revmtypes.EvmResult, error) {
	// save block context on evm
	excessBlobGas := evm.Context.BlobBaseFee.Uint64()
	number := evm.Context.BlockNumber
	evm.SetBlockNumber(number.Uint64())
	block := &revmtypes.Block{
		Number:        number.Bytes(),
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
	transaction := revmtypes.Transaction{
		Caller:         caller.Address().Bytes(),
		GasLimit:       msg.GasLimit,
		GasPrice:       msg.GasPrice.Bytes(),
		Nonce:          nil,
		TransactTo:     msg.To.Bytes(),
		Value:          msg.Value.Bytes(),
		Data:           msg.Data,
		GasPriorityFee: msg.GasTipCap.Bytes(),
		AccessList: func(accl types.AccessList) []*revmtypes.AccessListItem {
			result := make([]*revmtypes.AccessListItem, len(accl))
			for i, acc := range accl {
				storageKeys := make([]*revmtypes.StorageKey, len(acc.StorageKeys))
				for j, key := range acc.StorageKeys {
					storageKeys[j] = &revmtypes.StorageKey{Value: key.Bytes()}
				}
				result[i] = &revmtypes.AccessListItem{
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
