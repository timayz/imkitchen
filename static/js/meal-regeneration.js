/**
 * Meal Plan Regeneration Modal - JavaScript for CSP compliance
 * Story 3.7 - Review Action Item #1
 *
 * Handles modal interactions:
 * - Close modal on Cancel button or X button click
 * - Keyboard navigation (Escape to close, Enter to confirm)
 * - Focus management for accessibility
 * - Focus trap within modal (Tab/Shift+Tab)
 */

(function() {
    'use strict';

    // Modal close handler
    document.addEventListener('click', function(event) {
        const closeButton = event.target.closest('[data-close-modal]');
        if (closeButton) {
            const modal = document.getElementById('regenerate-modal');
            if (modal) {
                // Store the trigger element to return focus later
                const returnFocusElement = document.activeElement;
                modal.remove();

                // Return focus to the "Regenerate Meal Plan" button
                const regenerateButton = document.querySelector('[ts-req="/plan/regenerate/confirm"]');
                if (regenerateButton) {
                    regenerateButton.focus();
                }
            }
        }
    });

    // Keyboard navigation
    document.addEventListener('keydown', function(event) {
        const modal = document.getElementById('regenerate-modal');
        if (!modal) return;

        // Escape key closes modal
        if (event.key === 'Escape') {
            event.preventDefault();
            const closeButton = modal.querySelector('[data-close-modal]');
            if (closeButton) {
                closeButton.click();
            }
        }

        // Enter key submits form when focused on form elements (except textarea)
        if (event.key === 'Enter' && event.target.tagName !== 'TEXTAREA') {
            const form = modal.querySelector('form');
            if (form && document.activeElement !== form.querySelector('[data-close-modal]')) {
                // Only submit if not focused on cancel button
                event.preventDefault();
                form.querySelector('button[type="submit"]').click();
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

    // Auto-focus first input (reason field) when modal is inserted
    const observer = new MutationObserver(function(mutations) {
        mutations.forEach(function(mutation) {
            mutation.addedNodes.forEach(function(node) {
                if (node.id === 'regenerate-modal') {
                    // Focus the textarea (reason field) or first button
                    const reasonField = node.querySelector('textarea#regeneration_reason');
                    const firstFocusable = node.querySelector(
                        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
                    );

                    if (reasonField) {
                        reasonField.focus();
                    } else if (firstFocusable) {
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
})();
