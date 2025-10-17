/**
 * Meal Replacement Modal - JavaScript for CSP compliance
 * Story 3.6 - Review Action Item [M1]
 *
 * Handles modal interactions:
 * - Close modal on Cancel button or X button click
 * - Keyboard navigation (Escape to close)
 * - Focus management for accessibility
 * - Auto-dismiss toast notifications
 */

(function() {
    'use strict';

    // Modal close handler
    document.addEventListener('click', function(event) {
        const closeButton = event.target.closest('[data-close-modal]');
        if (closeButton) {
            const modal = document.getElementById('replace-modal');
            if (modal) {
                // Store the trigger element to return focus later
                const returnFocusElement = document.activeElement;
                modal.remove();

                // Return focus to the trigger button if it exists
                if (returnFocusElement && returnFocusElement.hasAttribute('ts-req')) {
                    returnFocusElement.focus();
                }
            }
        }
    });

    // Keyboard navigation
    document.addEventListener('keydown', function(event) {
        const modal = document.getElementById('replace-modal');
        if (!modal) return;

        // Escape key closes modal
        if (event.key === 'Escape') {
            event.preventDefault();
            const closeButton = modal.querySelector('[data-close-modal]');
            if (closeButton) {
                closeButton.click();
            }
        }

        // Tab key - trap focus within modal
        if (event.key === 'Tab') {
            const focusableElements = modal.querySelectorAll(
                'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
            );
            const firstFocusable = focusableElements[0];
            const lastFocusable = focusableElements[focusableElements.length - 1];

            if (event.shiftKey) {
                // Shift + Tab
                if (document.activeElement === firstFocusable) {
                    event.preventDefault();
                    lastFocusable.focus();
                }
            } else {
                // Tab
                if (document.activeElement === lastFocusable) {
                    event.preventDefault();
                    firstFocusable.focus();
                }
            }
        }
    });

    // Auto-focus first focusable element when modal is inserted
    const observer = new MutationObserver(function(mutations) {
        mutations.forEach(function(mutation) {
            mutation.addedNodes.forEach(function(node) {
                if (node.id === 'replace-modal') {
                    // Focus the first focusable element in the modal
                    const firstFocusable = node.querySelector(
                        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
                    );
                    if (firstFocusable) {
                        firstFocusable.focus();
                    }
                }
            });
        });
    });

    observer.observe(document.body, {
        childList: true,
        subtree: true
    });

    // Toast auto-dismiss handler
    document.addEventListener('DOMContentLoaded', function() {
        // Set up observer for dynamically added toasts
        const toastObserver = new MutationObserver(function(mutations) {
            mutations.forEach(function(mutation) {
                mutation.addedNodes.forEach(function(node) {
                    if (node.nodeType === 1 && node.hasAttribute('data-dismiss-after')) {
                        const dismissAfter = parseInt(node.getAttribute('data-dismiss-after'), 10);
                        if (dismissAfter > 0) {
                            setTimeout(function() {
                                if (node.parentNode) {
                                    node.remove();
                                }
                            }, dismissAfter);
                        }
                    }
                });
            });
        });

        toastObserver.observe(document.body, {
            childList: true,
            subtree: true
        });
    });
})();
