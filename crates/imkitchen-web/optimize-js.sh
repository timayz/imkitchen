#!/bin/bash

# JavaScript Optimization Script for imkitchen PWA
# Minifies and optimizes JavaScript files for production

STATIC_DIR="/home/snapiz/projects/github/snapiz/imkitchen/crates/imkitchen-web/static"
JS_DIR="$STATIC_DIR/js"

echo "Optimizing JavaScript files..."

# Check if terser is available for minification
if command -v terser >/dev/null 2>&1; then
    echo "Using Terser for JavaScript minification"
    
    # Create minified version of app.js
    terser "$JS_DIR/app.js" \
        --compress drop_console=true,drop_debugger=true,pure_funcs=['console.log','console.debug'] \
        --mangle \
        --output "$JS_DIR/app.min.js"
    
    echo "✓ Created app.min.js"
    
    # Create production versions with better compression
    terser "$JS_DIR/app.js" \
        --compress passes=2,drop_console=true,drop_debugger=true \
        --mangle \
        --format beautify=false \
        --output "$JS_DIR/app.prod.js"
    
    echo "✓ Created app.prod.js"
    
else
    echo "Terser not found. Creating manual optimization..."
    
    # Create a simplified version by removing comments and console logs
    grep -v "console\." "$JS_DIR/app.js" | \
    grep -v "^\s*\/\/" | \
    grep -v "^\s*\/\*" | \
    sed '/^\s*$/d' > "$JS_DIR/app.min.js"
    
    echo "✓ Created basic minified app.min.js"
fi

# Create lazy loading implementation
cat > "$JS_DIR/lazy-load.js" << 'EOF'
/**
 * Lazy Loading Implementation for imkitchen
 * Loads non-critical resources after initial page load
 */

// Lazy load images with intersection observer
function initImageLazyLoading() {
    if ('IntersectionObserver' in window) {
        const imageObserver = new IntersectionObserver((entries, observer) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    const img = entry.target;
                    if (img.dataset.src) {
                        img.src = img.dataset.src;
                        img.classList.remove('lazy');
                        observer.unobserve(img);
                    }
                }
            });
        });

        document.querySelectorAll('img[data-src]').forEach(img => {
            imageObserver.observe(img);
        });
    } else {
        // Fallback for browsers without IntersectionObserver
        document.querySelectorAll('img[data-src]').forEach(img => {
            img.src = img.dataset.src;
            img.classList.remove('lazy');
        });
    }
}

// Lazy load non-critical CSS
function loadCriticalCSS() {
    const nonCriticalCSS = [
        // Add paths to non-critical CSS files here
        // '/static/css/print.css',
        // '/static/css/animations.css'
    ];
    
    nonCriticalCSS.forEach(href => {
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = href;
        link.media = 'print';
        link.onload = function() { this.media = 'all'; };
        document.head.appendChild(link);
    });
}

// Initialize lazy loading after DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    // Small delay to prioritize critical rendering
    setTimeout(() => {
        initImageLazyLoading();
        loadCriticalCSS();
    }, 100);
});

// Export for module systems if needed
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { initImageLazyLoading, loadCriticalCSS };
}
EOF

echo "✓ Created lazy-load.js"

# Show file sizes for comparison
echo ""
echo "File sizes:"
ls -lh "$JS_DIR"/*.js | awk '{print $5 "\t" $9}'

echo ""
echo "JavaScript optimization complete!"
echo ""
echo "Usage in production:"
echo "1. Use app.min.js or app.prod.js instead of app.js"
echo "2. Include lazy-load.js for non-critical resource loading"
echo "3. Configure server to serve compressed versions with appropriate cache headers"