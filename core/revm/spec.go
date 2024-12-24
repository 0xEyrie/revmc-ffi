package revm

// SpecId represents the different Ethereum hard fork specifications
type SpecId uint8

const (
	FRONTIER         SpecId = 0
	FRONTIER_THAWING SpecId = 1
	HOMESTEAD        SpecId = 2
	DAO_FORK         SpecId = 3
	TANGERINE        SpecId = 4
	SPURIOUS_DRAGON  SpecId = 5
	BYZANTIUM        SpecId = 6
	CONSTANTINOPLE   SpecId = 7
	PETERSBURG       SpecId = 8
	ISTANBUL         SpecId = 9
	MUIR_GLACIER     SpecId = 10
	BERLIN           SpecId = 11
	LONDON           SpecId = 12
	ARROW_GLACIER    SpecId = 13
	GRAY_GLACIER     SpecId = 14
	MERGE            SpecId = 15
	SHANGHAI         SpecId = 16
	CANCUN           SpecId = 17
	PRAGUE           SpecId = 18
	OSAKA            SpecId = 19
	LATEST           SpecId = 255
)
