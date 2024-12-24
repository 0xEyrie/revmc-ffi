//go:build darwin

package revm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lrevmapi
import "C"
