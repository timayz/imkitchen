package services

import (
	"bytes"
	"context"
	"fmt"
	"image"
	"image/jpeg"
	"image/png"
	"io"
	"mime/multipart"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/nfnt/resize"
	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
)

type PhotoService interface {
	UploadRecipePhoto(file multipart.File, header *multipart.FileHeader, recipeID string) (string, error)
	DeleteRecipePhoto(photoURL string) error
	OptimizeImage(file multipart.File, maxWidth, maxHeight uint) ([]byte, error)
}

type photoService struct {
	minioClient *minio.Client
	bucketName  string
	publicURL   string
}

func NewPhotoService() (PhotoService, error) {
	// Get MinIO configuration from environment
	endpoint := getEnvDefault("MINIO_ENDPOINT", "localhost:9000")
	accessKey := getEnvDefault("MINIO_ACCESS_KEY", "minioadmin")
	secretKey := getEnvDefault("MINIO_SECRET_KEY", "minioadmin")
	bucketName := getEnvDefault("MINIO_BUCKET", "imkitchen-recipes")
	publicURL := getEnvDefault("MINIO_PUBLIC_URL", "http://localhost:9000")
	useSSL := getEnvDefault("MINIO_USE_SSL", "false") == "true"

	// Initialize MinIO client
	minioClient, err := minio.New(endpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(accessKey, secretKey, ""),
		Secure: useSSL,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create MinIO client: %w", err)
	}

	// Create bucket if it doesn't exist
	ctx := context.Background()
	err = minioClient.MakeBucket(ctx, bucketName, minio.MakeBucketOptions{})
	if err != nil {
		// Check if bucket already exists
		exists, existsErr := minioClient.BucketExists(ctx, bucketName)
		if existsErr != nil || !exists {
			return nil, fmt.Errorf("failed to create or check bucket: %w", err)
		}
	}

	// Set bucket policy to public read
	policy := fmt.Sprintf(`{
		"Version": "2012-10-17",
		"Statement": [
			{
				"Effect": "Allow",
				"Principal": "*",
				"Action": ["s3:GetObject"],
				"Resource": ["arn:aws:s3:::%s/recipe-photos/*"]
			}
		]
	}`, bucketName)

	err = minioClient.SetBucketPolicy(ctx, bucketName, policy)
	if err != nil {
		// Log warning but don't fail the service
		fmt.Printf("Warning: Could not set bucket policy: %v\n", err)
	}

	return &photoService{
		minioClient: minioClient,
		bucketName:  bucketName,
		publicURL:   publicURL,
	}, nil
}

func (s *photoService) UploadRecipePhoto(file multipart.File, header *multipart.FileHeader, recipeID string) (string, error) {
	// Validate file type
	contentType := header.Header.Get("Content-Type")
	if !s.isValidImageType(contentType) {
		return "", fmt.Errorf("invalid file type: %s. Only JPEG and PNG are supported", contentType)
	}

	// Validate file size (max 10MB)
	if header.Size > 10*1024*1024 {
		return "", fmt.Errorf("file too large: %d bytes. Maximum size is 10MB", header.Size)
	}

	// Reset file reader position
	file.Seek(0, 0)

	// Optimize image
	optimizedImage, err := s.OptimizeImage(file, 800, 600)
	if err != nil {
		return "", fmt.Errorf("failed to optimize image: %w", err)
	}

	// Generate unique filename
	filename := s.generatePhotoFilename(recipeID, filepath.Ext(header.Filename))
	objectPath := fmt.Sprintf("recipe-photos/%s", filename)

	// Upload to MinIO
	ctx := context.Background()
	reader := bytes.NewReader(optimizedImage)
	
	uploadInfo, err := s.minioClient.PutObject(ctx, s.bucketName, objectPath, reader, int64(len(optimizedImage)), minio.PutObjectOptions{
		ContentType: "image/jpeg", // Always save as JPEG after optimization
	})
	if err != nil {
		return "", fmt.Errorf("failed to upload image: %w", err)
	}

	// Generate public URL
	publicURL := fmt.Sprintf("%s/%s/%s", s.publicURL, s.bucketName, objectPath)
	
	// Log upload info
	fmt.Printf("Uploaded recipe photo: %s, size: %d bytes\n", uploadInfo.Key, uploadInfo.Size)

	return publicURL, nil
}

func (s *photoService) DeleteRecipePhoto(photoURL string) error {
	if photoURL == "" {
		return nil
	}

	// Extract object path from URL
	objectPath := s.extractObjectPathFromURL(photoURL)
	if objectPath == "" {
		return fmt.Errorf("invalid photo URL: %s", photoURL)
	}

	// Delete from MinIO
	ctx := context.Background()
	err := s.minioClient.RemoveObject(ctx, s.bucketName, objectPath, minio.RemoveObjectOptions{})
	if err != nil {
		return fmt.Errorf("failed to delete image: %w", err)
	}

	fmt.Printf("Deleted recipe photo: %s\n", objectPath)
	return nil
}

func (s *photoService) OptimizeImage(file multipart.File, maxWidth, maxHeight uint) ([]byte, error) {
	// Reset file reader position
	file.Seek(0, 0)

	// Decode image
	img, format, err := image.Decode(file)
	if err != nil {
		return nil, fmt.Errorf("failed to decode image: %w", err)
	}

	// Get original dimensions
	bounds := img.Bounds()
	originalWidth := uint(bounds.Dx())
	originalHeight := uint(bounds.Dy())

	// Calculate new dimensions while maintaining aspect ratio
	var newWidth, newHeight uint

	if originalWidth > maxWidth || originalHeight > maxHeight {
		// Calculate scaling factor
		widthScale := float64(maxWidth) / float64(originalWidth)
		heightScale := float64(maxHeight) / float64(originalHeight)
		scale := widthScale
		if heightScale < widthScale {
			scale = heightScale
		}

		newWidth = uint(float64(originalWidth) * scale)
		newHeight = uint(float64(originalHeight) * scale)
	} else {
		// Image is already within limits
		newWidth = originalWidth
		newHeight = originalHeight
	}

	// Resize image if needed
	var resizedImg image.Image
	if newWidth != originalWidth || newHeight != originalHeight {
		resizedImg = resize.Resize(newWidth, newHeight, img, resize.Lanczos3)
	} else {
		resizedImg = img
	}

	// Encode as JPEG with quality 85
	var buffer bytes.Buffer
	err = jpeg.Encode(&buffer, resizedImg, &jpeg.Options{Quality: 85})
	if err != nil {
		// If JPEG encoding fails and original was PNG, try PNG
		if format == "png" {
			buffer.Reset()
			err = png.Encode(&buffer, resizedImg)
			if err != nil {
				return nil, fmt.Errorf("failed to encode optimized image: %w", err)
			}
		} else {
			return nil, fmt.Errorf("failed to encode optimized image: %w", err)
		}
	}

	return buffer.Bytes(), nil
}

func (s *photoService) isValidImageType(contentType string) bool {
	validTypes := []string{
		"image/jpeg",
		"image/jpg",
		"image/png",
	}

	for _, validType := range validTypes {
		if contentType == validType {
			return true
		}
	}

	return false
}

func (s *photoService) generatePhotoFilename(recipeID string, extension string) string {
	timestamp := time.Now().Unix()
	uniqueID := uuid.New().String()[:8]
	
	// Normalize extension
	if extension == "" {
		extension = ".jpg"
	}
	if !strings.HasPrefix(extension, ".") {
		extension = "." + extension
	}
	extension = strings.ToLower(extension)
	
	// Replace .jpeg with .jpg for consistency
	if extension == ".jpeg" {
		extension = ".jpg"
	}
	
	return fmt.Sprintf("%s-%d-%s%s", recipeID, timestamp, uniqueID, extension)
}

func (s *photoService) extractObjectPathFromURL(photoURL string) string {
	// Extract object path from URL like: http://localhost:9000/bucket/recipe-photos/filename.jpg
	parts := strings.Split(photoURL, fmt.Sprintf("/%s/", s.bucketName))
	if len(parts) != 2 {
		return ""
	}
	return parts[1]
}