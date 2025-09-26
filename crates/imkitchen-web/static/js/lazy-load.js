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
