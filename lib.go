package revmcffi

import (
	"github.com/0xEyrie/revmc-ffi/core"
	types "github.com/rethmint/revm-api/types/go"
)

// VM struct is the core of initiavm.
type VM struct {
	Inner core.VM
}

type Compiler struct {
	Inner core.Compiler
}

func NewCompiler(threshold uint64) Compiler {
	inner := core.InitCompiler(threshold)
	return Compiler{inner}
}

func (compiler *Compiler) Destroy() {
	core.ReleaseCompiler(compiler.Inner)
}

// NewVm return VM instance
// handler
func NewVM(SPEC_ID uint8) VM {
	inner := core.NewVM(SPEC_ID)

	return VM{inner}
}

func NewVMWithCompiler(SPEC_ID uint8, compiler Compiler) VM {
	inner := core.NewVMWithCompiler(SPEC_ID, compiler.Inner)

	return VM{inner}
}

func (vm *VM) Destroy() {
	core.ReleaseVM(vm.Inner)
}

// ExecuteTx execute transaction based on revm
// for bootstrapping genesis
func (vm *VM) ExecuteTx(
	kvStore core.KVStore,
	block types.SerializedBlock,
	tx types.SerializedTransaction,
) (types.ExecutionResult, error) {
	res, err := core.ExecuteTx(
		vm.Inner,
		kvStore,
		&block,
		&tx,
	)
	if err != nil {
		return nil, err
	}

	return res, nil
}

func (vm *VM) QueryTx(
	kvStore core.,
	block types.SerializedBlock,
	tx types.SerializedTransaction,
) (types.ExecutionResult, error) {
	res, err := core.QueryTx(
		vm.Inner,
		kvStore,
		&block,
		&tx,
	)
	if err != nil {
		return nil, err
	}

	return res, nil
}
