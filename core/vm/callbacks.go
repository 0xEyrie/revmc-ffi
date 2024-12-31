package vm

// Check https://akrennmair.github.io/golang-cgo-slides/ to learn
// how this embedded C code works.

/*
#include "bindings.h"

// typedefs for _cgo functions (db)
typedef GoError (*commit_fn)(db_t *ptr, U8SliceView codes, U8SliceView storages, U8SliceView accounts, U8SliceView deletedAccounts, UnmanagedVector *errOut);
typedef GoError (*get_account_fn)(db_t *ptr, U8SliceView address, UnmanagedVector *result, UnmanagedVector *errOut);
typedef GoError (*get_code_by_hash_fn)(db_t *ptr, U8SliceView codeHash, UnmanagedVector *result, UnmanagedVector *errOut);
typedef GoError (*get_storage_fn)(db_t *ptr, U8SliceView address, U8SliceView key, UnmanagedVector *result, UnmanagedVector *errOut);
typedef GoError (*get_block_hash_fn)(db_t *ptr, uint64_t blockNumber, UnmanagedVector *result, UnmanagedVector *errOut);

// forward declarations (db)
GoError cCommit_cgo(db_t *ptr, U8SliceView codes, U8SliceView storages, U8SliceView accounts, U8SliceView deletedAccounts, UnmanagedVector *errOut);
GoError cGetAccount_cgo(db_t *ptr, U8SliceView address, UnmanagedVector *result, UnmanagedVector *errOut);
GoError cGetCodeByHash_cgo(db_t *ptr, U8SliceView codeHash, UnmanagedVector *result, UnmanagedVector *errOut);
GoError cGetStorage_cgo(db_t *ptr, U8SliceView address, U8SliceView key, UnmanagedVector *result, UnmanagedVector *errOut);
GoError cGetBlockHash_cgo(db_t *ptr, uint64_t blockNumber, UnmanagedVector *result, UnmanagedVector *errOut);
*/
import "C"

import (
	"log"
	"runtime/debug"
	"unsafe"

	state "github.com/0xEyrie/revmffi/core/state"
)

// Note: we have to include all exports in the same file (at least since they both import bindings.h),
// or get odd cgo build errors about duplicate definitions

func recoverPanic(ret *C.GoError) {
	if rec := recover(); rec != nil {
		log.Printf("Panic in Go callback: %#v\n", rec)
		debug.PrintStack()
		*ret = C.GoError_Panic
	}
}

type Gas = uint64

// GasMeter is a copy of an interface declaration from cosmos-sdk
// https://github.com/cosmos/cosmos-sdk/blob/18890a225b46260a9adc587be6fa1cc2aff101cd/store/types/gas.go#L34
type GasMeter interface {
	GasConsumed() Gas
}

/****** DB ********/

var db_vtable = C.Db_vtable{
	commit:           (C.commit_fn)(C.cCommit_cgo),
	get_account:      (C.get_account_fn)(C.cGetAccount_cgo),
	get_code_by_hash: (C.get_code_by_hash_fn)(C.cGetAccount_cgo),
	get_storage:      (C.get_storage_fn)(C.cGetStorage_cgo),
	get_block_hash:   (C.get_block_hash_fn)(C.cGetBlockHash_cgo),
}

type DBState struct {
	State state.ExtendedStateDB
}

// use this to create C.Db in two steps, so the pointer lives as long as the calling stack
//
//	// then pass db into some FFI function
func buildDBState(state state.ExtendedStateDB) DBState {
	return DBState{
		State: state,
	}
}

// contract: original pointer/struct referenced must live longer than C.Db struct
// since this is only used internally, we can verify the code that this is the case
func buildDB(state *DBState) C.Db {
	return C.Db{
		state:  (*C.db_t)(unsafe.Pointer(state)),
		vtable: db_vtable,
	}
}

//export cCommit
func cCommit(ptr *C.db_t, storages C.U8SliceView, accounts C.U8SliceView, deletedAccounts C.U8SliceView, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	statedb := *(*state.ExtendedStateDB)(unsafe.Pointer(ptr))
	v0 := copyU8Slice(storages)
	v1 := copyU8Slice(accounts)
	v2 := copyU8Slice(deletedAccounts)

	statedb.UpdateAndCommit(v0, v1, v2)

	return C.GoError_None
}

//export cBasic
func cBasic(ptr *C.db_t, address C.U8SliceView, account *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || account == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*account).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	statedb := *(*state.ExtendedStateDB)(unsafe.Pointer(ptr))
	addr := copyU8Slice(address)
	v := statedb.Basic(addr)

	// v will equal nil when the key is missing
	// https://github.com/cosmos/cosmos-sdk/blob/1083fa948e347135861f88e07ec76b0314296832/store/types/store.go#L174
	*account = newUnmanagedVector(v)

	return C.GoError_None
}

//export cGetCodeByHash
func cGetCodeByHash(ptr *C.db_t, codeHash C.U8SliceView, code *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || code == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*code).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	statedb := *(*state.ExtendedStateDB)(unsafe.Pointer(ptr))
	k := copyU8Slice(codeHash)
	v := statedb.GetCodeByHash(k)

	*code = newUnmanagedVector(v)

	return C.GoError_None
}

//export cGetStorage
func cGetStorage(ptr *C.db_t, address C.U8SliceView, storageKey C.U8SliceView, storageValue *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || storageValue == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*storageValue).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	statedb := *(*state.ExtendedStateDB)(unsafe.Pointer(ptr))
	addr := copyU8Slice(address)
	sk := copyU8Slice(storageKey)
	v := statedb.GetStorage(addr, sk)

	*storageValue = newUnmanagedVector(v)

	return C.GoError_None
}

//export cGetBlockHash
func cGetBlockHash(ptr *C.db_t, blockNumber C.uint64_t, blockHash *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || blockHash == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*blockHash).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	statedb := *(*state.ExtendedStateDB)(unsafe.Pointer(ptr))
	bh := statedb.GetBlockHash(uint64(blockNumber))

	*blockHash = newUnmanagedVector(bh)

	return C.GoError_None
}
