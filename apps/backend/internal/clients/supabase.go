package clients

import (
	"os"
	"fmt"
	"log"
	
	"github.com/supabase-community/supabase-go"
)

type SupabaseClient struct {
	Client *supabase.Client
}

// NewSupabaseClient creates a new Supabase client instance
func NewSupabaseClient() (*SupabaseClient, error) {
	supabaseURL := os.Getenv("SUPABASE_URL")
	if supabaseURL == "" {
		return nil, fmt.Errorf("SUPABASE_URL environment variable is required")
	}

	supabaseKey := os.Getenv("SUPABASE_ANON_KEY")
	if supabaseKey == "" {
		return nil, fmt.Errorf("SUPABASE_ANON_KEY environment variable is required")
	}

	client, err := supabase.NewClient(supabaseURL, supabaseKey, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create Supabase client: %w", err)
	}

	log.Printf("Supabase client initialized successfully")
	
	return &SupabaseClient{
		Client: client,
	}, nil
}

// GetServiceClient returns a Supabase client with service role key for admin operations
func NewSupabaseServiceClient() (*SupabaseClient, error) {
	supabaseURL := os.Getenv("SUPABASE_URL")
	if supabaseURL == "" {
		return nil, fmt.Errorf("SUPABASE_URL environment variable is required")
	}

	serviceRoleKey := os.Getenv("SUPABASE_SERVICE_ROLE_KEY")
	if serviceRoleKey == "" {
		return nil, fmt.Errorf("SUPABASE_SERVICE_ROLE_KEY environment variable is required")
	}

	client, err := supabase.NewClient(supabaseURL, serviceRoleKey, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create Supabase service client: %w", err)
	}

	log.Printf("Supabase service client initialized successfully")
	
	return &SupabaseClient{
		Client: client,
	}, nil
}