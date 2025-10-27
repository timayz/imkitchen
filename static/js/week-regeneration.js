/**
 * Week Regeneration Modal - JavaScript for Story 9.5
 *
 * Handles week regeneration modal interactions:
 * - AC-9.5.3: Open modal for single week regeneration
 * - AC-9.5.4: Open modal for all future weeks regeneration
 * - AC-9.5.6: Confirm triggers POST with TwinSpark
 * - AC-9.5.7: Loading spinner during regeneration
 * - AC-9.5.8: Success - calendar updates
 * - AC-9.5.9: Error - display toast
 * - Keyboard navigation (Escape to close, Tab for focus trap)
 * - Focus management for accessibility
 */

(function() {
    'use strict';

    /**
     * AC-9.5.3: Open modal for single week regeneration
     * @param {string} weekId - Week ID to regenerate
     * @param {number} weekNumber - Week number (1, 2, 3, etc.)
     * @param {string} startDate - Week start date (YYYY-MM-DD)
     * @param {string} endDate - Week end date (YYYY-MM-DD)
     */
    window.openRegenerateWeekModal = function(weekId, weekNumber, startDate, endDate) {
        const modal = document.getElementById('week-regeneration-modal');
        const description = document.getElementById('regeneration-modal-description');
        const typeInput = document.getElementById('regeneration-type');
        const weekIdInput = document.getElementById('regeneration-week-id');

        if (!modal || !description || !typeInput || !weekIdInput) {
            console.error('Week regeneration modal elements not found');
            return;
        }

        // Set modal content for single week
        description.textContent = `Replace meals for Week ${weekNumber} (${startDate} - ${endDate})?`;
        typeInput.value = 'single';
        weekIdInput.value = weekId;

        // Show modal
        modal.classList.remove('hidden');

        // Focus first button (Confirm)
        setTimeout(() => {
            const confirmBtn = document.getElementById('confirm-regeneration-btn');
            if (confirmBtn) confirmBtn.focus();
        }, 100);
    };

    /**
     * AC-9.5.4: Open modal for all future weeks regeneration
     * @param {number} futureWeeksCount - Number of future weeks to regenerate
     */
    window.openRegenerateAllModal = function(futureWeeksCount) {
        const modal = document.getElementById('week-regeneration-modal');
        const description = document.getElementById('regeneration-modal-description');
        const typeInput = document.getElementById('regeneration-type');
        const weekIdInput = document.getElementById('regeneration-week-id');

        if (!modal || !description || !typeInput || !weekIdInput) {
            console.error('Week regeneration modal elements not found');
            return;
        }

        // Set modal content for all future weeks
        description.textContent = `Regenerate ${futureWeeksCount} future week${futureWeeksCount > 1 ? 's' : ''}? Your current week will be preserved.`;
        typeInput.value = 'all';
        weekIdInput.value = ''; // Not needed for "all" regeneration

        // Show modal
        modal.classList.remove('hidden');

        // Focus first button (Confirm)
        setTimeout(() => {
            const confirmBtn = document.getElementById('confirm-regeneration-btn');
            if (confirmBtn) confirmBtn.focus();
        }, 100);
    };

    /**
     * Close modal
     */
    function closeModal() {
        const modal = document.getElementById('week-regeneration-modal');
        if (modal) {
            modal.classList.add('hidden');
        }

        // Return focus to the trigger button (best practice for accessibility)
        // Focus will automatically return to the button that opened the modal
    }

    /**
     * AC-9.5.7: Show loading spinner
     */
    function showSpinner() {
        const spinner = document.getElementById('regeneration-loading-spinner');
        if (spinner) {
            spinner.classList.remove('hidden');
        }
    }

    /**
     * AC-9.5.7: Hide loading spinner
     */
    function hideSpinner() {
        const spinner = document.getElementById('regeneration-loading-spinner');
        if (spinner) {
            spinner.classList.add('hidden');
        }
    }

    /**
     * AC-9.5.9: Show error toast
     * @param {string} message - Error message
     */
    function showErrorToast(message) {
        // Create toast element
        const toast = document.createElement('div');
        toast.className = 'fixed top-4 right-4 bg-red-600 text-white px-6 py-4 rounded-lg shadow-lg z-50 flex items-center gap-3 max-w-md';
        toast.setAttribute('role', 'alert');
        toast.setAttribute('aria-live', 'assertive');

        toast.innerHTML = `
            <svg class="w-6 h-6 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
            <span class="font-medium">${message}</span>
            <button class="ml-auto text-white/80 hover:text-white" onclick="this.parentElement.remove()" aria-label="Close">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
            </button>
        `;

        document.body.appendChild(toast);

        // Auto-dismiss after 5 seconds
        setTimeout(() => {
            toast.remove();
        }, 5000);
    }

    /**
     * AC-9.5.6: Handle confirm button click - trigger POST with TwinSpark
     */
    document.addEventListener('click', function(event) {
        const confirmBtn = event.target.closest('#confirm-regeneration-btn');
        if (confirmBtn) {
            event.preventDefault();

            const typeInput = document.getElementById('regeneration-type');
            const weekIdInput = document.getElementById('regeneration-week-id');

            if (!typeInput) {
                console.error('Regeneration type input not found');
                return;
            }

            const regenerationType = typeInput.value;
            let url;

            if (regenerationType === 'single') {
                const weekId = weekIdInput.value;
                if (!weekId) {
                    console.error('Week ID not provided for single week regeneration');
                    return;
                }
                url = `/plan/week/${weekId}/regenerate`;
            } else if (regenerationType === 'all') {
                url = '/plan/regenerate-all-future';
            } else {
                console.error('Invalid regeneration type:', regenerationType);
                return;
            }

            // Close modal
            closeModal();

            // Show spinner (AC-9.5.7)
            showSpinner();

            // AC-9.5.6: Make POST request with TwinSpark-like behavior
            fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                    'Accept': 'text/html',
                },
                credentials: 'same-origin',
            })
            .then(response => {
                hideSpinner();

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }

                // AC-9.5.8: Success - reload page to show updated calendar
                // TwinSpark would normally swap innerHTML, but for simplicity we reload
                window.location.href = '/plan';
            })
            .catch(error => {
                hideSpinner();
                console.error('Regeneration failed:', error);
                // AC-9.5.9: Show error toast
                showErrorToast('Failed to regenerate. Please try again.');
            });
        }
    });

    /**
     * Modal close handlers
     */
    document.addEventListener('click', function(event) {
        const closeButton = event.target.closest('[data-close-week-regeneration-modal]');
        if (closeButton) {
            closeModal();
        }
    });

    /**
     * Keyboard navigation
     */
    document.addEventListener('keydown', function(event) {
        const modal = document.getElementById('week-regeneration-modal');
        if (!modal || modal.classList.contains('hidden')) return;

        // Escape key closes modal
        if (event.key === 'Escape') {
            event.preventDefault();
            closeModal();
        }

        // Enter key submits form (confirm)
        if (event.key === 'Enter' && event.target.tagName !== 'TEXTAREA') {
            event.preventDefault();
            const confirmBtn = document.getElementById('confirm-regeneration-btn');
            if (confirmBtn) {
                confirmBtn.click();
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
})();
