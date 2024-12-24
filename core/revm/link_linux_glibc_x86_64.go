//go:build linux && !muslc && amd64

package revm

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lrevmapi.x86_64
import "C"
