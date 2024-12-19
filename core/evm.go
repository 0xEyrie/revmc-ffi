package core

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"

	"github.com/0xEyrie/revmc-ffi/state"
	revmtypes "github.com/0xEyrie/revmc-ffi/types"
	"google.golang.org/protobuf/proto"
)

type EVM struct {
	evm_ptr *C.evm_t
	aot     bool
}

// DestroyVM call ffi(`release_vm`) to release vm instance
func DestroyVM(vm EVM) {
	C.free_vm(vm.evm_ptr, C.bool(vm.aot))
}

// NewVM call ffi(`init_vm`) to initialize vm instance
func NewVM(SPEC_ID uint8) EVM {
	return EVM{
		evm_ptr: C.new_vm(cu8(SPEC_ID)),
		aot:     false,
	}
}

// NewVMWithCompiler call ffi(`init_aot_vm`) to initialize vm instance
func NewVMWithCompiler(SPEC_ID uint8, compiler Compiler) EVM {
	return EVM{
		evm_ptr: C.new_vm_with_compiler(cu8(SPEC_ID), compiler.ptr),
		aot:     true,
	}
}

// ExecuteTx call ffi(`execute_tx`) to execute transaction
func ExecuteTx(
	vm EVM,
	store state.StateDB,
	block *[]byte,
	tx *[]byte,
) (*revmtypes.EvmResult, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	blockBytesSliceView := makeView(*block)
	defer runtime.KeepAlive(blockBytesSliceView)
	txByteSliceView := makeView(*tx)
	defer runtime.KeepAlive(txByteSliceView)

	errmsg := uninitializedUnmanagedVector()
	res, err := C.execute_tx(vm.evm_ptr, C.bool(vm.aot), db, blockBytesSliceView, txByteSliceView, &errmsg)
	if err != nil && err.(syscall.Errno) != C.Success {
		// ignore the opereation times out error
		errno, ok := err.(syscall.Errno)
		if ok && errno == syscall.ETIMEDOUT || errno == syscall.ENOENT {
			return decodeEvmResult(res)
		}
		return &revmtypes.EvmResult{}, errorWithMessage(err, errmsg)
	}

	return decodeEvmResult(res)
}

// SimulateTx call ffi(`simulate_tx`) to simulate transaction
func SimulateTx(
	vm EVM,
	store state.StateDB,
	block *[]byte,
	tx *[]byte,
) (*revmtypes.EvmResult, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	blockBytesSliceView := makeView(*block)
	defer runtime.KeepAlive(blockBytesSliceView)
	txByteSliceView := makeView(*tx)
	defer runtime.KeepAlive(txByteSliceView)

	errmsg := uninitializedUnmanagedVector()
	res, err := C.simulate_tx(vm.evm_ptr, C.bool(vm.aot), db, blockBytesSliceView, txByteSliceView, &errmsg)
	if err != nil && err.(syscall.Errno) != C.Success {
		// ignore the operation timed out error
		errno, ok := err.(syscall.Errno)
		if ok && errno == syscall.ETIMEDOUT || errno == syscall.ENOENT {
			return decodeEvmResult(res)
		}
		return &revmtypes.EvmResult{}, errorWithMessage(err, errmsg)
	}

	return decodeEvmResult(res)
}

type Compiler struct {
	ptr *C.compiler_t
}

func ReleaseCompiler(compiler Compiler) {
	C.free_compiler(compiler.ptr)
}

func InitCompiler(threshold uint64) Compiler {
	return Compiler{
		ptr: C.new_compiler(C.uint64_t(threshold)),
	}
}

func decodeEvmResult(res C.UnmanagedVector) (*revmtypes.EvmResult, error) {
	vec := copyAndDestroyUnmanagedVector(res)
	var result revmtypes.EvmResult
	err := proto.Unmarshal(vec, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}
