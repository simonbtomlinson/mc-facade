package main

import (
	"C"
	"context"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/compute/v1"
)

//export StartVM
func StartVM(projectID, zone string) string {
	ctx := context.Background()
	client, err := google.DefaultClient(ctx, compute.ComputeScope)
	if err != nil {
		return err.Error()
	}
	computeService, err := compute.New(client)
	_ = computeService // Not used yet, will be later
	if err != nil {
		return err.Error()
	}
	return ""
}

// Needed for Go to be able to compile this
func main() {}
