package core

/*
#include "bindings.h"
#include <stdio.h>

// imports (db)
GoError cCommit(db_t *ptr, U8SliceView codes, U8SliceView storages, U8SliceView accounts, U8SliceView deletedAccounts, UnmanagedVector *errOut);
GoError cGetAccount(db_t *ptr, U8SliceView address, UnmanagedVector *result, UnmanagedVector *errOut);
GoError cGetCodeByHash(db_t *ptr, U8SliceView codeHash, UnmanagedVector *result, UnmanagedVector *errOut);
GoError cGetStorage(db_t *ptr, U8SliceView address, U8SliceView key, UnmanagedVector *result, UnmanagedVector *errOut);
GoError cGetBlockHash(db_t *ptr, uint64_t blockNumber, UnmanagedVector *result, UnmanagedVector *errOut);

// Gateway functions (db)
GoError cCommit_cgo(db_t *ptr, U8SliceView codes, U8SliceView storages, U8SliceView accounts, U8SliceView deletedAccounts, UnmanagedVector *errOut) {
	return cCommit(ptr, codes, storages, accounts, deletedAccounts, errOut);
}
GoError cGetAccount_cgo(db_t *ptr, U8SliceView address, UnmanagedVector *result, UnmanagedVector *errOut) {
	return cGetAccount(ptr, address, result, errOut);
}
GoError cGetCodeByHash_cgo(db_t *ptr, U8SliceView codeHash, UnmanagedVector *result, UnmanagedVector *errOut) {
	return cGetCodeByHash(ptr, codeHash, result, errOut);
}
GoError cGetStorage_cgo(db_t *ptr, U8SliceView address, U8SliceView key, UnmanagedVector *result, UnmanagedVector *errOut) {
	return cGetStorage(ptr, address, key, result, errOut);
}
GoError cGetBlockHash_cgo(db_t *ptr, uint64_t blockNumber, UnmanagedVector *result, UnmanagedVector *errOut) {
	return cGetBlockHash(ptr, blockNumber, result, errOut);
}
*/
import "C"

// We need these gateway functions to allow calling back to a go function from the c code.
// At least I didn't discover a cleaner way.
// Also, this needs to be in a different file than `callbacks.go`, as we cannot create functions
// in the same file that has //export directives. Only import header types
