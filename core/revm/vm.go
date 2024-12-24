package revm

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"

	revmtypes "github.com/0xEyrie/revmc-ffi/types"
	"google.golang.org/protobuf/proto"
)

// EVM represents an Ethereum Virtual Machine instance
type EVM struct {
	evm_ptr     *C.evm_t
	StateDB     StateDB
	hasCompiler bool
}

// DestroyVM releases the VM instance
func DestroyVM(vm EVM) {
	C.free_vm(vm.evm_ptr, C.bool(vm.hasCompiler))
}

// NewVM initializes a new VM instance
func NewVM(statedb StateDB, spec SpecId) EVM {
	return EVM{
		evm_ptr:     C.new_vm(cu8(spec)),
		StateDB:     statedb,
		hasCompiler: false,
	}
}

// NewVMWithCompiler initializes a new VM instance with AOT compiler
func NewVMWithCompiler(statedb StateDB, thershold uint64, maxConcurrentSize uint, spec SpecId) EVM {
	return EVM{
		evm_ptr:     C.new_vm_with_compiler(cu8(spec), cu64(thershold), cusize(maxConcurrentSize)),
		StateDB:     statedb,
		hasCompiler: true,
	}
}

// ExecuteTx executes a transaction on the VM
func (vm *EVM) Execute(
	block *[]byte,
	tx *[]byte,
) (*revmtypes.EvmResult, error) {
	var err error
	dbState := buildDBState(vm.StateDB)
	db := buildDB(&dbState)

	blockBytesSliceView := makeView(*block)
	defer runtime.KeepAlive(blockBytesSliceView)
	txByteSliceView := makeView(*tx)
	defer runtime.KeepAlive(txByteSliceView)

	errmsg := uninitializedUnmanagedVector()
	res, err := C.execute_tx(vm.evm_ptr, C.bool(vm.hasCompiler), db, blockBytesSliceView, txByteSliceView, &errmsg)
	if err != nil && err.(syscall.Errno) != C.Success {
		// ignore the opereation times out error
		errno, ok := err.(syscall.Errno)
		if ok && errno == syscall.ETIMEDOUT || errno == syscall.ENOENT {
			return unmarshalEvmResult(res)
		}
		return &revmtypes.EvmResult{}, errorWithMessage(err, errmsg)
	}

	return unmarshalEvmResult(res)
}

// SimulateTx simulates a transaction on the VM
func (vm *EVM) simulate(
	block *[]byte,
	tx *[]byte,
) (*revmtypes.EvmResult, error) {
	var err error
	dbState := buildDBState(vm.StateDB)
	db := buildDB(&dbState)

	blockBytesSliceView := makeView(*block)
	defer runtime.KeepAlive(blockBytesSliceView)
	txByteSliceView := makeView(*tx)
	defer runtime.KeepAlive(txByteSliceView)

	errmsg := uninitializedUnmanagedVector()
	res, err := C.simulate_tx(vm.evm_ptr, C.bool(vm.hasCompiler), db, blockBytesSliceView, txByteSliceView, &errmsg)
	if err != nil && err.(syscall.Errno) != C.Success {
		// ignore the operation timed out error
		errno, ok := err.(syscall.Errno)
		if ok && errno == syscall.ETIMEDOUT || errno == syscall.ENOENT {
			return unmarshalEvmResult(res)
		}
		return &revmtypes.EvmResult{}, errorWithMessage(err, errmsg)
	}

	return unmarshalEvmResult(res)
}

// unmarshalEvmResult decodes the EVM result from the unmanaged vector
func unmarshalEvmResult(res C.UnmanagedVector) (*revmtypes.EvmResult, error) {
	vec := copyAndDestroyUnmanagedVector(res)
	var result revmtypes.EvmResult
	err := proto.Unmarshal(vec, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}
