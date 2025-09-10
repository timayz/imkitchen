import { launchCamera, launchImageLibrary, ImagePickerResponse, ImagePickerOptions } from 'react-native-image-picker';
import { Alert } from 'react-native';
import { RecipeClient } from '@imkitchen/api-client';

export interface PhotoUploadResult {
  success: boolean;
  url?: string;
  error?: string;
}

export interface PhotoPickerOptions {
  allowsEditing?: boolean;
  quality?: number;
  maxWidth?: number;
  maxHeight?: number;
}

export class PhotoService {
  private recipeClient: RecipeClient;

  constructor(recipeClient: RecipeClient) {
    this.recipeClient = recipeClient;
  }

  // Show photo picker options (camera or library)
  showPhotoPicker(options?: PhotoPickerOptions): Promise<string | null> {
    return new Promise((resolve) => {
      Alert.alert(
        'Select Photo',
        'Choose how you want to add a photo',
        [
          {
            text: 'Camera',
            onPress: () => this.openCamera(options).then(resolve),
          },
          {
            text: 'Photo Library',
            onPress: () => this.openLibrary(options).then(resolve),
          },
          {
            text: 'Cancel',
            style: 'cancel',
            onPress: () => resolve(null),
          },
        ]
      );
    });
  }

  // Open camera for photo capture
  openCamera(options?: PhotoPickerOptions): Promise<string | null> {
    return new Promise((resolve) => {
      const pickerOptions: ImagePickerOptions = {
        mediaType: 'photo',
        quality: options?.quality || 0.8,
        maxWidth: options?.maxWidth || 800,
        maxHeight: options?.maxHeight || 600,
        includeBase64: false,
        allowsEditing: options?.allowsEditing || true,
      };

      launchCamera(pickerOptions, (response: ImagePickerResponse) => {
        this.handleImagePickerResponse(response, resolve);
      });
    });
  }

  // Open photo library
  openLibrary(options?: PhotoPickerOptions): Promise<string | null> {
    return new Promise((resolve) => {
      const pickerOptions: ImagePickerOptions = {
        mediaType: 'photo',
        quality: options?.quality || 0.8,
        maxWidth: options?.maxWidth || 800,
        maxHeight: options?.maxHeight || 600,
        includeBase64: false,
        allowsEditing: options?.allowsEditing || true,
        selectionLimit: 1,
      };

      launchImageLibrary(pickerOptions, (response: ImagePickerResponse) => {
        this.handleImagePickerResponse(response, resolve);
      });
    });
  }

  // Handle image picker response
  private handleImagePickerResponse(
    response: ImagePickerResponse,
    resolve: (value: string | null) => void
  ) {
    if (response.didCancel) {
      resolve(null);
      return;
    }

    if (response.errorMessage) {
      Alert.alert('Error', response.errorMessage);
      resolve(null);
      return;
    }

    if (response.assets && response.assets[0]) {
      const asset = response.assets[0];
      if (asset.uri) {
        resolve(asset.uri);
      } else {
        Alert.alert('Error', 'Failed to get image URI');
        resolve(null);
      }
    } else {
      resolve(null);
    }
  }

  // Upload photo to recipe
  async uploadRecipePhoto(recipeId: string, imageUri: string): Promise<PhotoUploadResult> {
    try {
      // Create FormData
      const formData = new FormData();
      
      // Determine file type from URI
      const fileType = imageUri.toLowerCase().includes('.png') ? 'image/png' : 'image/jpeg';
      const fileName = `recipe-${recipeId}-${Date.now()}.${fileType.includes('png') ? 'png' : 'jpg'}`;

      formData.append('photo', {
        uri: imageUri,
        type: fileType,
        name: fileName,
      } as any);

      // Upload to backend
      const response = await fetch(`${this.recipeClient['baseUrl']}/api/v1/recipes/${recipeId}/photo`, {
        method: 'POST',
        body: formData,
        headers: {
          'Content-Type': 'multipart/form-data',
          'Authorization': `Bearer ${this.recipeClient['authToken']}`,
        },
      });

      if (!response.ok) {
        const error = await response.json();
        return {
          success: false,
          error: error.error || `HTTP ${response.status}: ${response.statusText}`,
        };
      }

      const result = await response.json();
      return {
        success: true,
        url: result.url,
      };
    } catch (error) {
      console.error('Photo upload error:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Upload failed',
      };
    }
  }

  // Delete recipe photo
  async deleteRecipePhoto(recipeId: string): Promise<PhotoUploadResult> {
    try {
      const response = await fetch(`${this.recipeClient['baseUrl']}/api/v1/recipes/${recipeId}/photo`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${this.recipeClient['authToken']}`,
        },
      });

      if (!response.ok) {
        const error = await response.json();
        return {
          success: false,
          error: error.error || `HTTP ${response.status}: ${response.statusText}`,
        };
      }

      return { success: true };
    } catch (error) {
      console.error('Photo delete error:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Delete failed',
      };
    }
  }

  // Validate image file
  validateImageFile(uri: string, maxSizeBytes = 10 * 1024 * 1024): Promise<boolean> {
    return new Promise((resolve) => {
      // Basic URI validation
      if (!uri || !uri.startsWith('file://') && !uri.startsWith('content://')) {
        resolve(false);
        return;
      }

      // For now, assume valid - more sophisticated validation could be added
      resolve(true);
    });
  }

  // Get optimized image dimensions
  getOptimizedDimensions(
    originalWidth: number,
    originalHeight: number,
    maxWidth = 800,
    maxHeight = 600
  ): { width: number; height: number } {
    if (originalWidth <= maxWidth && originalHeight <= maxHeight) {
      return { width: originalWidth, height: originalHeight };
    }

    const widthScale = maxWidth / originalWidth;
    const heightScale = maxHeight / originalHeight;
    const scale = Math.min(widthScale, heightScale);

    return {
      width: Math.round(originalWidth * scale),
      height: Math.round(originalHeight * scale),
    };
  }
}