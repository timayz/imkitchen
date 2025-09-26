#!/bin/bash

# imkitchen PWA Icon Generation Script
# This script generates placeholder icons for the PWA in different sizes

ICON_DIR="$(dirname "$0")"
cd "$ICON_DIR"

# Create SVG base icon
cat > base-icon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <!-- Green background -->
  <rect width="512" height="512" fill="#10b981" rx="64"/>
  
  <!-- Kitchen/Chef hat icon -->
  <g transform="translate(256,256)">
    <!-- Chef hat -->
    <circle cx="0" cy="-60" r="45" fill="white"/>
    <circle cx="-25" cy="-80" r="20" fill="white"/>
    <circle cx="25" cy="-80" r="20" fill="white"/>
    <circle cx="-35" cy="-50" r="15" fill="white"/>
    <circle cx="35" cy="-50" r="15" fill="white"/>
    
    <!-- Hat band -->
    <rect x="-50" y="-25" width="100" height="15" fill="white"/>
    
    <!-- Kitchen utensil - spoon -->
    <ellipse cx="-15" cy="20" rx="8" ry="25" fill="white"/>
    <rect x="-18" y="45" width="6" height="40" fill="white"/>
    
    <!-- Kitchen utensil - fork -->
    <rect x="10" y="45" width="3" height="40" fill="white"/>
    <rect x="17" y="45" width="3" height="40" fill="white"/>
    <rect x="10" y="45" width="10" height="15" fill="white"/>
  </g>
  
  <!-- App name hint -->
  <text x="256" y="450" text-anchor="middle" font-family="Arial, sans-serif" font-size="36" font-weight="bold" fill="white">ik</text>
</svg>
EOF

echo "Generated base SVG icon"

# Generate different sizes using ImageMagick (if available) or provide instructions
if command -v convert >/dev/null 2>&1; then
    echo "ImageMagick found. Generating PNG icons..."
    
    # Generate required PWA icon sizes
    convert base-icon.svg -resize 16x16 icon-16x16.png
    convert base-icon.svg -resize 32x32 icon-32x32.png
    convert base-icon.svg -resize 144x144 icon-144x144.png
    convert base-icon.svg -resize 192x192 icon-192x192.png
    convert base-icon.svg -resize 512x512 icon-512x512.png
    
    echo "Generated all required icon sizes:"
    echo "✓ icon-16x16.png"
    echo "✓ icon-32x32.png" 
    echo "✓ icon-144x144.png"
    echo "✓ icon-192x192.png"
    echo "✓ icon-512x512.png"
else
    echo "ImageMagick not found. Please install ImageMagick and run:"
    echo "  sudo apt-get install imagemagick  # Ubuntu/Debian"
    echo "  brew install imagemagick          # macOS"
    echo ""
    echo "Then run this script again to generate PNG icons from base-icon.svg"
    echo ""
    echo "Alternatively, use any SVG to PNG converter to create:"
    echo "- icon-16x16.png (16x16 pixels)"
    echo "- icon-32x32.png (32x32 pixels)" 
    echo "- icon-144x144.png (144x144 pixels)"
    echo "- icon-192x192.png (192x192 pixels)"
    echo "- icon-512x512.png (512x512 pixels)"
fi

echo "Icon generation complete!"