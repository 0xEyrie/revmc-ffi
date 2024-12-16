package revmcffi_test

const EOF_FIB_BIN = "0xef0001010004020001001103000101450400000000800002608060405234e100055f6080ee005f80fdef000101000c0200030043001d005e0400680000800004020100030101000460806040526004361015e100035f80fd5f3560e01c6361047ff41415e1ffee34e1001d6020600319360112e1000f6020600435e30002604051908152f35f80fd5f80fd908101809111e10001e4634e487b7160e01b5f52601160045260245ffd80155f14e10003505fe4600181145f14e10004506001e45f198101818111e1002ae30002906001198101908111e10008e3000290e30001e4634e487b7160e01b5f52601160045260245ffd634e487b7160e01b5f52601160045260245ffda3646970667358221220d894df004ff6699df9241f03ac960821d7bc31aad25f77ac1a2e267e21039a506c6578706572696d656e74616cf564736f6c637827302e382e32372d646576656c6f702e323032342e382e352b636f6d6d69742e38386366363036300066"

// type TestContract struct {
// 	name      string
// 	aot       bool
// 	caller    common.Address
// 	bin       []byte
// 	abi       *abi.ABI
// 	txdata    []byte
// 	calldatas []CallData
// 	repeat    int
// }

// type CallData struct {
// 	name     string
// 	calldata []byte
// 	expected types.Success
// }

// const CANCUN uint8 = 17
// const OSAKA uint8 = 19

// func setupTest(t *testing.T, aot bool, caller common.Address) (revm.VM, *testutils.MockKVStore) {
// 	kvStore := testutils.NewMockKVStore()

// 	var compiler revm.Compiler
// 	var vm revm.VM

// 	if aot {
// 		compiler = revm.NewCompiler(1)
// 		vm = revm.NewVMWithCompiler(OSAKA, compiler)
// 	} else {
// 		vm = revm.NewVM(OSAKA)
// 	}

// 	t.Cleanup(func() {
// 		vm.Destroy()
// 		if aot {
// 			compiler.Destroy()
// 		}
// 	})

// 	testutils.Faucet(kvStore, caller, big.NewInt(1000000000000))

// 	return vm, kvStore
// }

// func TestErc20WithoutAOT(t *testing.T) {
// 	aot := false
// 	Erc20CA(t, aot)
// }

// func TestErc20WithAOT(t *testing.T) {
// 	aot := true
// 	Erc20CA(t, aot)
// }

// func Erc20CA(t *testing.T, aot bool) {
// 	caller := common.HexToAddress("0xe100713fc15400d1e94096a545879e7c647001e0")

// 	erc20abi, _ := Erc20.Erc20MetaData.GetAbi()
// 	erc20bin, _ := hexutil.Decode(Erc20.Erc20Bin)
// 	packedData, _ := erc20abi.Constructor.Inputs.Pack("Mock", "Mock")
// 	txdata := append(erc20bin, packedData...)

// 	mintData, _ := erc20abi.Pack("mint", caller, big.NewInt(1000))
// 	mintCallData := CallData{
// 		name:     "mint()",
// 		calldata: mintData,
// 		expected: types.Success{
// 			Reason:      "Stop",
// 			GasUsed:     0x11d96,
// 			GasRefunded: 0x0,
// 			Logs: []types.Log{
// 				{
// 					Address: common.HexToAddress("0xeC30481c768e48D34Ea8fc2bEbcfeAddEBA6bfA4"),
// 					Data: types.LogData{
// 						Topics: []common.Hash{
// 							common.HexToHash("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
// 							common.HexToHash("0x0000000000000000000000000000000000000000000000000000000000000000"),
// 							common.HexToHash("0x000000000000000000000000e100713fc15400d1e94096a545879e7c647001e0"),
// 						}, Data: []uint8{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0xe8},
// 					},
// 				},
// 			}, Output: types.Output{
// 				DeployedAddress: common.Address{},
// 				Output:          []uint8{},
// 			},
// 		},
// 	}

// 	recipientAddr := common.HexToAddress("0x20")
// 	transferData, _ := erc20abi.Pack("transfer", recipientAddr, big.NewInt(100))
// 	transferCallData := CallData{
// 		name:     "transfer()",
// 		calldata: transferData,
// 		expected: types.Success{
// 			Reason:      "Return",
// 			GasUsed:     0xdb79,
// 			GasRefunded: 0x0,
// 			Logs: []types.Log{
// 				{
// 					Address: common.HexToAddress("0xeC30481c768e48D34Ea8fc2bEbcfeAddEBA6bfA4"),
// 					Data: types.LogData{
// 						Topics: []common.Hash{
// 							common.HexToHash("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
// 							common.HexToHash("0x000000000000000000000000e100713fc15400d1e94096a545879e7c647001e0"),
// 							common.HexToHash("0x0000000000000000000000000000000000000000000000000000000000000020"),
// 						},
// 						Data: []uint8{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x64},
// 					},
// 				},
// 			}, Output: types.Output{
// 				DeployedAddress: common.Address{},
// 				Output:          []uint8{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1},
// 			},
// 		},
// 	}

// 	ca := TestContract{
// 		name:   "Erc20",
// 		aot:    aot,
// 		caller: caller,
// 		bin:    erc20bin,
// 		abi:    erc20abi,
// 		txdata: txdata,
// 		calldatas: []CallData{
// 			mintCallData,
// 			transferCallData,
// 		},
// 		repeat: 1,
// 	}

// 	Ca(t, ca)
// }

// func TestFibWithoutAOT(t *testing.T) {
// 	aot := false
// 	Fib(t, aot)
// }

// func TestFibWithAOT(t *testing.T) {
// 	aot := true
// 	Fib(t, aot)
// }

// func Fib(t *testing.T, aot bool) {
// 	caller := common.HexToAddress("0xe100713fc15400d1e94096a545879e7c647001e0")

// 	fibbin, err := hexutil.Decode(fibca.FibonacciBin)
// 	require.NoError(t, err)
// 	fibabi, err := fibca.FibonacciMetaData.GetAbi()
// 	require.NoError(t, err)
// 	txdata, err := hexutil.Decode(fibca.FibonacciBin)
// 	require.NoError(t, err)

// 	fibData, err := fibabi.Pack("fibonacci", big.NewInt(25))
// 	require.NoError(t, err)

// 	fibCallData := CallData{
// 		name:     "fibonacci()",
// 		calldata: fibData,
// 		expected: types.Success{
// 			Reason:      "Return",
// 			GasUsed:     0x2bff4bd,
// 			GasRefunded: 0x0,
// 			Logs:        []types.Log{},
// 			Output: types.Output{
// 				DeployedAddress: common.Address{},
// 				Output: []uint8{
// 					0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x25, 0x11,
// 				},
// 			},
// 		},
// 	}

// 	ca := TestContract{
// 		name:   "Fibonacci",
// 		aot:    aot,
// 		caller: caller,
// 		bin:    fibbin,
// 		abi:    fibabi,
// 		txdata: txdata,
// 		calldatas: []CallData{
// 			fibCallData,
// 		},
// 		repeat: 5,
// 	}

// 	Ca(t, ca)
// }

// func TestEofFibWithoutAOT(t *testing.T) {
// 	aot := false
// 	FibEof(t, aot)
// }

// func TestEofFibWithAOT(t *testing.T) {
// 	aot := true
// 	FibEof(t, aot)
// }

// func FibEof(t *testing.T, aot bool) {
// 	caller := common.HexToAddress("0xe100713fc15400d1e94096a545879e7c647001e0")

// 	fibbin, err := hexutil.Decode(EOF_FIB_BIN)
// 	require.NoError(t, err)
// 	fibabi, err := fibca.FibonacciMetaData.GetAbi()
// 	require.NoError(t, err)
// 	txdata, err := hexutil.Decode(EOF_FIB_BIN)
// 	require.NoError(t, err)

// 	fibData, err := fibabi.Pack("fibonacci", big.NewInt(25))
// 	require.NoError(t, err)

// 	fibCallData := CallData{
// 		name:     "fibonacci()",
// 		calldata: fibData,
// 		expected: types.Success{
// 			Reason:      "Return",
// 			GasUsed:     0x1318b20,
// 			GasRefunded: 0x0,
// 			Logs:        []types.Log{},
// 			Output: types.Output{
// 				DeployedAddress: common.Address{},
// 				Output:          []uint8{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x25, 0x11},
// 			},
// 		},
// 	}

// 	ca := TestContract{
// 		name:   "Fibonacci",
// 		aot:    aot,
// 		caller: caller,
// 		bin:    fibbin,
// 		abi:    fibabi,
// 		txdata: txdata,
// 		calldatas: []CallData{
// 			fibCallData,
// 		},
// 		repeat: 5,
// 	}

// 	Ca(t, ca)
// }

// func Ca(t *testing.T, ca TestContract) {
// 	fmt.Printf("Testing contract %s...\n", ca.name)

// 	vm, kvStore := setupTest(t, ca.aot, ca.caller)

// 	createTx := testutils.MockTx(ca.caller, common.Address{}, ca.txdata, 0)
// 	block := testutils.MockBlock(1)
// 	res, err := vm.ExecuteTx(kvStore, block.ToSerialized(), createTx.ToSerialized())
// 	require.NoError(t, err)
// 	result, err := res.ProcessExecutionResult()
// 	require.NoError(t, err)
// 	createRes, ok := result.(types.Success)
// 	require.True(t, ok)
// 	deployedAddr := createRes.Output.DeployedAddress

// 	nonce := uint64(1)
// 	for repeat := 0; repeat < ca.repeat; repeat++ {
// 		for i := 0; i < len(ca.calldatas); i++ {
// 			calldata := ca.calldatas[i]
// 			testTx := testutils.MockTx(ca.caller, deployedAddr, calldata.calldata, nonce)

// 			start := time.Now()

// 			res, err = vm.ExecuteTx(kvStore, block.ToSerialized(), testTx.ToSerialized())
// 			require.NoError(t, err)

// 			result, err = res.ProcessExecutionResult()
// 			require.NoError(t, err)

// 			callRes, ok := result.(types.Success)
// 			require.True(t, ok)

// 			require.Equal(t, calldata.expected, callRes)

// 			elapsed := time.Since(start)
// 			t.Logf("%s: Test %s: Call %d execution time: %v", ca.name, calldata.name, i+1, elapsed)

// 			time.Sleep(1 * time.Second)
// 			nonce++
// 		}
// 	}
// }
