package vm

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"

	"github.com/0xEyrie/revmffi/core/state"
	"github.com/0xEyrie/revmffi/core/types"
	"google.golang.org/protobuf/proto"
)

// EVM represents an Ethereum Virtual Machine instance
type EVM struct {
	evm_ptr     *C.evm_t
	StateDB     state.ExtendedStateDB
	hasCompiler bool
}

// DestroyVM releases the VM instance
func DestroyVM(vm EVM) {
	C.free_vm(vm.evm_ptr, C.bool(vm.hasCompiler))
}

// NewEVM initializes a new VM instance
func NewEVM(statedb state.ExtendedStateDB, spec SpecId) EVM {
	return EVM{
		evm_ptr:     C.new_vm(cu8(spec)),
		StateDB:     statedb,
		hasCompiler: false,
	}
}

// NewEVMWithCompiler initializes a new VM instance with AOT compiler
func NewEVMWithCompiler(statedb state.ExtendedStateDB, thershold uint64, maxConcurrentSize uint, spec SpecId) EVM {
	return EVM{
		evm_ptr:     C.new_vm_with_compiler(cu8(spec), cu64(thershold), cusize(maxConcurrentSize)),
		StateDB:     statedb,
		hasCompiler: true,
	}
}

// `Execute` executes a transaction on the VM
func (evm *EVM) Execute(
	block *[]byte,
	tx *[]byte,
) (*types.EvmResult, error) {
	var err error
	dbState := buildDBState(evm.StateDB)
	db := buildDB(&dbState)

	blockBytesSliceView := makeView(*block)
	defer runtime.KeepAlive(blockBytesSliceView)
	txByteSliceView := makeView(*tx)
	defer runtime.KeepAlive(txByteSliceView)

	errmsg := uninitializedUnmanagedVector()
	res, err := C.execute_tx(evm.evm_ptr, C.bool(evm.hasCompiler), db, blockBytesSliceView, txByteSliceView, &errmsg)
	if err != nil && err.(syscall.Errno) != C.Success {
		// ignore the opereation times out error
		errno, ok := err.(syscall.Errno)
		if ok && errno == syscall.ETIMEDOUT || errno == syscall.ENOENT {
			return unmarshalResult(res)
		}
		return &types.EvmResult{}, errorWithMessage(err, errmsg)
	}

	return unmarshalResult(res)
}

// `Simulate` simulates a transaction on the VM
func (evm *EVM) simulate(
	block *[]byte,
	tx *[]byte,
) (*types.EvmResult, error) {
	var err error
	dbState := buildDBState(evm.StateDB)
	db := buildDB(&dbState)

	blockBytesSliceView := makeView(*block)
	defer runtime.KeepAlive(blockBytesSliceView)
	txByteSliceView := makeView(*tx)
	defer runtime.KeepAlive(txByteSliceView)

	errmsg := uninitializedUnmanagedVector()
	res, err := C.simulate_tx(evm.evm_ptr, C.bool(evm.hasCompiler), db, blockBytesSliceView, txByteSliceView, &errmsg)
	if err != nil && err.(syscall.Errno) != C.Success {
		// ignore the operation timed out error
		errno, ok := err.(syscall.Errno)
		if ok && errno == syscall.ETIMEDOUT || errno == syscall.ENOENT {
			return unmarshalResult(res)
		}
		return &types.EvmResult{}, errorWithMessage(err, errmsg)
	}

	return unmarshalResult(res)
}

// unmarshalResult decodes the EVM result from the unmanaged vector
func unmarshalResult(res C.UnmanagedVector) (*types.EvmResult, error) {
	vec := copyAndDestroyUnmanagedVector(res)
	var result types.EvmResult
	err := proto.Unmarshal(vec, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}
