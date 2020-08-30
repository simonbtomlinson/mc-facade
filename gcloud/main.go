package main

import "C"

//export DoubleString
func DoubleString(input string) *C.char {
	return C.CString(input + input)
}

// Needed for Go to be able to compile this
func main() {}
