package types

import (
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/holiman/uint256"
)

func (x *Account) Into() ([]byte, types.StateAccount) {
	var stateacc types.StateAccount
	balance := new(uint256.Int).SetBytes(x.Balance)
	stateacc.Balance = balance

	stateacc.Nonce = x.Nonce
	stateacc.CodeHash = x.CodeHash

	return x.Code, stateacc
}
