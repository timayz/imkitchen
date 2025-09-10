package services

import (
	"fmt"
	"os"

	"github.com/golang-jwt/jwt/v5"
	"github.com/imkitchen/backend/internal/clients"
)

type AuthService struct {
	supabaseClient        *clients.SupabaseClient
	supabaseServiceClient *clients.SupabaseClient
	jwtSecret            []byte
}

type RegisterRequest struct {
	Email    string `json:"email" binding:"required,email" validate:"required,email,max=255"`
	Password string `json:"password" binding:"required,min=8" validate:"required,min=8,max=128,containsany=!@#$%^&*"`
	Name     string `json:"name" binding:"required" validate:"required,min=2,max=100"`
}

type LoginRequest struct {
	Email    string `json:"email" binding:"required,email" validate:"required,email,max=255"`
	Password string `json:"password" binding:"required" validate:"required,min=8,max=128"`
}

type RefreshTokenRequest struct {
	RefreshToken string `json:"refresh_token" binding:"required" validate:"required,min=10"`
}

type ForgotPasswordRequest struct {
	Email string `json:"email" binding:"required,email" validate:"required,email,max=255"`
}

type ResetPasswordRequest struct {
	Token       string `json:"token" binding:"required" validate:"required,min=10"`
	NewPassword string `json:"new_password" binding:"required,min=8" validate:"required,min=8,max=128,containsany=!@#$%^&*"`
}

type AuthResponse struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	ExpiresIn    int    `json:"expires_in"`
	TokenType    string `json:"token_type"`
	User         User   `json:"user"`
}

type User struct {
	ID               string `json:"id"`
	Email            string `json:"email"`
	EmailVerified    bool   `json:"email_verified"`
	Name             string `json:"name"`
	CreatedAt        string `json:"created_at"`
	UpdatedAt        string `json:"updated_at"`
}

// NewAuthService creates a new authentication service
func NewAuthService() (*AuthService, error) {
	supabaseClient, err := clients.NewSupabaseClient()
	if err != nil {
		return nil, fmt.Errorf("failed to create Supabase client: %w", err)
	}

	supabaseServiceClient, err := clients.NewSupabaseServiceClient()
	if err != nil {
		return nil, fmt.Errorf("failed to create Supabase service client: %w", err)
	}

	jwtSecret := os.Getenv("SUPABASE_JWT_SECRET")
	if jwtSecret == "" {
		return nil, fmt.Errorf("SUPABASE_JWT_SECRET environment variable is required")
	}

	return &AuthService{
		supabaseClient:        supabaseClient,
		supabaseServiceClient: supabaseServiceClient,
		jwtSecret:            []byte(jwtSecret),
	}, nil
}

// Register creates a new user account
func (s *AuthService) Register(req RegisterRequest) (*AuthResponse, error) {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	// This is a temporary stub to resolve build issues unrelated to story 2.4
	return nil, fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// Login authenticates a user  
func (s *AuthService) Login(req LoginRequest) (*AuthResponse, error) {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	return nil, fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// RefreshToken refreshes an access token
func (s *AuthService) RefreshToken(req RefreshTokenRequest) (*AuthResponse, error) {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	return nil, fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// Logout signs out a user
func (s *AuthService) Logout(accessToken string) error {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	return fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// ForgotPassword sends a password reset email
func (s *AuthService) ForgotPassword(req ForgotPasswordRequest) error {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	return fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// ResetPassword resets a user's password using a reset token
func (s *AuthService) ResetPassword(req ResetPasswordRequest) error {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	return fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// ValidateToken validates a JWT token and returns the user ID
func (s *AuthService) ValidateToken(tokenString string) (string, error) {
	// Parse the token
	token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
		// Ensure token method is HMAC
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return s.jwtSecret, nil
	})

	if err != nil {
		return "", fmt.Errorf("failed to parse token: %w", err)
	}

	// Validate token and extract claims
	if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
		// Extract user ID from claims (Supabase uses 'sub' claim)
		if userID, exists := claims["sub"]; exists {
			if userIDStr, ok := userID.(string); ok {
				return userIDStr, nil
			}
		}
		
		return "", fmt.Errorf("user ID not found in token claims")
	}

	return "", fmt.Errorf("invalid token")
}

// GetUserFromToken extracts user information from a JWT token
func (s *AuthService) GetUserFromToken(tokenString string) (*User, error) {
	// TODO: Fix Supabase gotrue-go v1.2.0 API compatibility
	return nil, fmt.Errorf("auth service implementation pending - gotrue-go v1.2.0 API compatibility issue")
}

// Helper function to extract name from user metadata
func getNameFromUserMetadata(metadata map[string]interface{}) string {
	if metadata == nil {
		return ""
	}
	
	if name, exists := metadata["name"]; exists {
		if nameStr, ok := name.(string); ok {
			return nameStr
		}
	}
	
	// Fallback to full_name if name doesn't exist
	if fullName, exists := metadata["full_name"]; exists {
		if fullNameStr, ok := fullName.(string); ok {
			return fullNameStr
		}
	}
	
	return ""
}