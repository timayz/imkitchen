/**
 * imkitchen App JavaScript
 * Custom functionality for PWA and responsive features
 */

// App initialization
document.addEventListener('DOMContentLoaded', function() {
    console.log('imkitchen app initialized');
    
    // Initialize PWA features
    initializePWA();
    
    // Initialize touch enhancements
    initializeTouchEnhancements();
    
    // Initialize accessibility features
    initializeAccessibility();
});

/**
 * PWA Initialization
 */
function initializePWA() {
    // Handle PWA install prompt
    let deferredPrompt;
    
    window.addEventListener('beforeinstallprompt', (e) => {
        console.log('PWA install prompt available');
        e.preventDefault();
        deferredPrompt = e;
        
        // Show custom install button if available
        const installButton = document.getElementById('pwa-install-btn');
        if (installButton) {
            installButton.style.display = 'block';
            installButton.addEventListener('click', () => {
                deferredPrompt.prompt();
                deferredPrompt.userChoice.then((choiceResult) => {
                    if (choiceResult.outcome === 'accepted') {
                        console.log('User accepted the PWA install prompt');
                    }
                    deferredPrompt = null;
                    installButton.style.display = 'none';
                });
            });
        }
    });
    
    // Handle app installed
    window.addEventListener('appinstalled', () => {
        console.log('PWA was installed');
        const installButton = document.getElementById('pwa-install-btn');
        if (installButton) {
            installButton.style.display = 'none';
        }
    });
}

/**
 * Touch Enhancement for Mobile
 */
function initializeTouchEnhancements() {
    // Ensure minimum touch targets (44px) for interactive elements
    const interactiveElements = document.querySelectorAll('button, a, input, select, textarea');
    
    interactiveElements.forEach(element => {
        const computedStyle = window.getComputedStyle(element);
        const height = parseFloat(computedStyle.height);
        const minHeight = parseFloat(computedStyle.minHeight);
        
        // Ensure minimum 44px touch target
        if (height < 44 && minHeight < 44) {
            element.style.minHeight = '44px';
            element.style.display = element.style.display || 'inline-flex';
            element.style.alignItems = 'center';
        }
    });
    
    // Add touch feedback for buttons
    const buttons = document.querySelectorAll('button, .btn');
    buttons.forEach(button => {
        button.addEventListener('touchstart', function() {
            this.classList.add('opacity-80');
        }, { passive: true });
        
        button.addEventListener('touchend', function() {
            setTimeout(() => {
                this.classList.remove('opacity-80');
            }, 150);
        }, { passive: true });
    });
}

/**
 * Accessibility Enhancements
 */
function initializeAccessibility() {
    // Mobile menu functionality (click and keyboard navigation)
    const mobileMenuButton = document.getElementById('mobile-menu-button');
    const mobileMenu = document.getElementById('mobile-menu');
    
    if (mobileMenuButton && mobileMenu) {
        // Handle mobile menu button click
        mobileMenuButton.addEventListener('click', function() {
            const isExpanded = mobileMenuButton.getAttribute('aria-expanded') === 'true';
            
            // Toggle menu visibility
            mobileMenu.classList.toggle('hidden');
            
            // Update aria-expanded
            mobileMenuButton.setAttribute('aria-expanded', !isExpanded);
            
            // Toggle hamburger/close icon
            const icon = mobileMenuButton.querySelector('svg');
            if (icon) {
                if (isExpanded) {
                    // Show hamburger
                    icon.innerHTML = '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>';
                } else {
                    // Show close (X)
                    icon.innerHTML = '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>';
                }
            }
        });
        
        // Close mobile menu when clicking outside
        document.addEventListener('click', function(event) {
            if (!mobileMenuButton.contains(event.target) && !mobileMenu.contains(event.target)) {
                mobileMenu.classList.add('hidden');
                mobileMenuButton.setAttribute('aria-expanded', 'false');
                
                // Reset to hamburger icon
                const icon = mobileMenuButton.querySelector('svg');
                if (icon) {
                    icon.innerHTML = '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>';
                }
            }
        });
        
        // Handle escape key to close menu
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape' && !mobileMenu.classList.contains('hidden')) {
                mobileMenu.classList.add('hidden');
                mobileMenuButton.setAttribute('aria-expanded', 'false');
                mobileMenuButton.focus();
                
                // Reset to hamburger icon
                const icon = mobileMenuButton.querySelector('svg');
                if (icon) {
                    icon.innerHTML = '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>';
                }
            }
        });
        
        // Trap focus within mobile menu when open
        const focusableElements = mobileMenu.querySelectorAll(
            'a, button, input, textarea, select, [tabindex]:not([tabindex="-1"])'
        );
        
        if (focusableElements.length > 0) {
            const firstFocusable = focusableElements[0];
            const lastFocusable = focusableElements[focusableElements.length - 1];
            
            mobileMenu.addEventListener('keydown', function(e) {
                if (e.key === 'Tab') {
                    if (e.shiftKey) {
                        // Shift + Tab
                        if (document.activeElement === firstFocusable) {
                            e.preventDefault();
                            lastFocusable.focus();
                        }
                    } else {
                        // Tab
                        if (document.activeElement === lastFocusable) {
                            e.preventDefault();
                            firstFocusable.focus();
                        }
                    }
                }
            });
        }
    }
    
}

/**
 * Screen reader announcement utility
 * Announces dynamic content changes to screen readers
 */
function announceToScreenReader(message) {
    if (!message) return;
    
    const announcement = document.createElement('div');
    announcement.setAttribute('aria-live', 'polite');
    announcement.setAttribute('aria-atomic', 'true');
    announcement.setAttribute('class', 'sr-only');
    announcement.textContent = message;
    
    document.body.appendChild(announcement);
    
    // Remove after 1 second to keep DOM clean
    setTimeout(() => {
        if (announcement.parentNode) {
            document.body.removeChild(announcement);
        }
    }, 1000);
}

/**
 * Utility Functions
 */

// Check if device supports touch
function isTouchDevice() {
    return (('ontouchstart' in window) ||
           (navigator.maxTouchPoints > 0) ||
           (navigator.msMaxTouchPoints > 0));
}

// Get current viewport size category
function getViewportCategory() {
    const width = window.innerWidth;
    if (width < 640) return 'mobile';
    if (width < 768) return 'sm';
    if (width < 1024) return 'md';
    return 'lg';
}

// Debounce function for resize events
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// Handle viewport changes
window.addEventListener('resize', debounce(() => {
    const category = getViewportCategory();
    document.body.setAttribute('data-viewport', category);
    
    // Close mobile menu on resize to larger viewport
    if (category !== 'mobile') {
        const mobileMenu = document.getElementById('mobile-menu');
        const mobileMenuButton = document.getElementById('mobile-menu-button');
        if (mobileMenu && mobileMenuButton) {
            mobileMenu.classList.add('hidden');
            mobileMenuButton.setAttribute('aria-expanded', 'false');
        }
    }
}, 250));

// Set initial viewport category
document.body.setAttribute('data-viewport', getViewportCategory());

// Add touch class to body if touch device
if (isTouchDevice()) {
    document.body.classList.add('touch-device');
}

// Export utilities for use by other scripts
window.imkitchen = {
    announceToScreenReader: announceToScreenReader,
    isTouchDevice: isTouchDevice,
    getViewportCategory: getViewportCategory,
    debounce: debounce
};