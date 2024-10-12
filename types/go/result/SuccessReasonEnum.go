// Code generated by the FlatBuffers compiler. DO NOT EDIT.

package result

import "strconv"

type SuccessReasonEnum int8

const (
	SuccessReasonEnumStop              SuccessReasonEnum = 0
	SuccessReasonEnumReturn            SuccessReasonEnum = 1
	SuccessReasonEnumSelfDestruct      SuccessReasonEnum = 2
	SuccessReasonEnumEofReturnContract SuccessReasonEnum = 3
)

var EnumNamesSuccessReasonEnum = map[SuccessReasonEnum]string{
	SuccessReasonEnumStop:              "Stop",
	SuccessReasonEnumReturn:            "Return",
	SuccessReasonEnumSelfDestruct:      "SelfDestruct",
	SuccessReasonEnumEofReturnContract: "EofReturnContract",
}

var EnumValuesSuccessReasonEnum = map[string]SuccessReasonEnum{
	"Stop":              SuccessReasonEnumStop,
	"Return":            SuccessReasonEnumReturn,
	"SelfDestruct":      SuccessReasonEnumSelfDestruct,
	"EofReturnContract": SuccessReasonEnumEofReturnContract,
}

func (v SuccessReasonEnum) String() string {
	if s, ok := EnumNamesSuccessReasonEnum[v]; ok {
		return s
	}
	return "SuccessReasonEnum(" + strconv.FormatInt(int64(v), 10) + ")"
}
