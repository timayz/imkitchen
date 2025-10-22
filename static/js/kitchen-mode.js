// Kitchen Mode - Story 5.6: Kitchen-Friendly Display Modes
// Handles: Toggle UI, Wake Lock API, localStorage persistence, step-by-step navigation

(function() {
    'use strict';

    // Wake Lock API state
    let wakeLock = null;

    // Kitchen Mode state management
    class KitchenModeManager {
        constructor() {
            this.enabled = this.loadPreference();
            this.toggleButton = null;
            this.exitButton = null;
            this.wakeLockIndicator = null;

            this.init();
        }

        init() {
            // Check if kitchen mode should be enabled via URL parameter
            const urlParams = new URLSearchParams(window.location.search);
            const kitchenModeParam = urlParams.get('kitchen_mode');

            // Apply Kitchen Mode if enabled from localStorage OR URL parameter
            if (this.enabled || kitchenModeParam === 'true') {
                this.applyKitchenMode();
                // If enabled via URL param but not in localStorage, enable it
                if (kitchenModeParam === 'true' && !this.enabled) {
                    this.enabled = true;
                    this.savePreference(true);
                    this.requestWakeLock();
                }
            }

            // Create toggle UI
            this.createToggleButton();
            this.createExitButton();
            this.createWakeLockIndicator();

            // Initialize step-by-step navigation if kitchen mode is active
            if (this.isKitchenModeActive()) {
                this.initStepByStepNavigation();
            }
        }

        // AC7: Load preference from localStorage
        /**
         * Loads Kitchen Mode preference from localStorage with error handling
         * @returns {boolean} true if Kitchen Mode should be enabled, false otherwise
         */
        loadPreference() {
            try {
                return localStorage.getItem('kitchen_mode_enabled') === 'true';
            } catch (e) {
                console.warn('Failed to load Kitchen Mode preference from localStorage:', e);
                return false;
            }
        }

        // AC7: Save preference to localStorage
        /**
         * Saves Kitchen Mode preference to localStorage with error handling
         * @param {boolean} enabled - Whether Kitchen Mode is enabled
         */
        savePreference(enabled) {
            try {
                if (enabled) {
                    localStorage.setItem('kitchen_mode_enabled', 'true');
                } else {
                    localStorage.removeItem('kitchen_mode_enabled');
                }
            } catch (e) {
                console.warn('Failed to save Kitchen Mode preference to localStorage:', e);
                // Continue operation even if localStorage fails (e.g., privacy mode, quota exceeded)
            }
        }

        /**
         * Checks if Kitchen Mode is currently active
         * @returns {boolean} true if Kitchen Mode is active
         */
        isKitchenModeActive() {
            return document.body.classList.contains('kitchen-mode');
        }

        // AC1: Setup toggle button from template
        /**
         * Sets up the Kitchen Mode toggle button (already in template)
         * Adds click listener and updates aria-checked based on current state
         */
        createToggleButton() {
            // Find the toggle button in the template (recipe detail page)
            const toggleButton = document.querySelector('[data-testid="kitchen-mode-toggle"]');
            if (!toggleButton) return; // Not on recipe detail page

            // Set initial aria-checked state
            toggleButton.setAttribute('aria-checked', this.isKitchenModeActive() ? 'true' : 'false');

            // Add click listener
            toggleButton.addEventListener('click', () => this.toggle());

            this.toggleButton = toggleButton;
        }

        // AC8: Create exit button (visible only in kitchen mode)
        /**
         * Creates the Exit Kitchen Mode button
         * Visible only when Kitchen Mode is active
         */
        createExitButton() {
            const exitButton = document.createElement('button');
            exitButton.setAttribute('data-testid', 'kitchen-mode-exit');
            exitButton.setAttribute('aria-label', 'Exit Kitchen Mode and return to normal view');

            exitButton.className = 'fixed top-4 right-4 z-50 bg-red-600 text-white rounded-lg shadow-lg hover:bg-red-700 transition-colors touch-target flex items-center gap-2 px-6 py-4 font-bold text-xl';

            exitButton.innerHTML = `
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
                <span>Exit Kitchen Mode</span>
            `;

            exitButton.style.display = this.isKitchenModeActive() ? 'flex' : 'none';
            exitButton.addEventListener('click', () => this.disable());

            document.body.appendChild(exitButton);
            this.exitButton = exitButton;
        }

        // AC6: Create wake lock indicator
        /**
         * Creates the visual wake lock indicator badge
         * Shows at bottom of screen when wake lock is active
         */
        createWakeLockIndicator() {
            const indicator = document.createElement('div');
            indicator.setAttribute('data-testid', 'wake-lock-indicator');
            indicator.setAttribute('aria-live', 'polite');
            indicator.setAttribute('aria-label', 'Screen wake lock is active');

            indicator.className = 'fixed bottom-4 left-1/2 transform -translate-x-1/2 bg-green-600 text-white rounded-full px-4 py-2 shadow-lg text-sm font-medium flex items-center gap-2';

            indicator.innerHTML = `
                <svg class="w-4 h-4 animate-pulse" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <circle cx="12" cy="12" r="10"></circle>
                </svg>
                <span>Screen awake</span>
            `;

            indicator.style.display = 'none';
            document.body.appendChild(indicator);
            this.wakeLockIndicator = indicator;
        }

        // AC1: Toggle kitchen mode on/off
        /**
         * Toggles Kitchen Mode on or off
         */
        toggle() {
            if (this.isKitchenModeActive()) {
                this.disable();
            } else {
                this.enable();
            }
        }

        /**
         * Enables Kitchen Mode
         * - Applies kitchen-mode class to body
         * - Saves preference to localStorage
         * - Requests wake lock
         * - Initializes step-by-step navigation
         * - Preserves scroll position
         */
        enable() {
            // AC8: Preserve scroll position
            const scrollY = window.scrollY;

            // Apply kitchen-mode class
            this.applyKitchenMode();

            // AC7: Save preference
            this.savePreference(true);

            // AC6: Request wake lock
            this.requestWakeLock();

            // Update toggle button aria-checked state
            if (this.toggleButton) {
                this.toggleButton.setAttribute('aria-checked', 'true');
            }

            // Show exit button
            if (this.exitButton) {
                this.exitButton.style.display = 'flex';
            }

            // Initialize step-by-step navigation
            this.initStepByStepNavigation();

            // AC8: Restore scroll position
            window.scrollTo(0, scrollY);
        }

        /**
         * Disables Kitchen Mode
         * - Removes kitchen-mode class from body
         * - Clears localStorage preference
         * - Releases wake lock
         * - Destroys step-by-step navigation
         * - Preserves scroll position
         */
        disable() {
            // AC8: Preserve scroll position
            const scrollY = window.scrollY;

            // Remove kitchen-mode class
            document.body.classList.remove('kitchen-mode');

            // AC7: Clear preference
            this.savePreference(false);

            // AC6: Release wake lock
            this.releaseWakeLock();

            // Update toggle button aria-checked state
            if (this.toggleButton) {
                this.toggleButton.setAttribute('aria-checked', 'false');
            }

            // Hide exit button
            if (this.exitButton) {
                this.exitButton.style.display = 'none';
            }

            // Destroy step-by-step navigation
            this.destroyStepByStepNavigation();

            // AC8: Restore scroll position
            window.scrollTo(0, scrollY);
        }

        /**
         * Applies kitchen-mode class to body element
         */
        applyKitchenMode() {
            document.body.classList.add('kitchen-mode');
        }

        // AC6: Request Wake Lock API
        /**
         * Requests Wake Lock to prevent screen from sleeping
         * Includes automatic re-request on page visibility change
         */
        async requestWakeLock() {
            // Feature detection
            if (!('wakeLock' in navigator)) {
                console.log('Wake Lock API not supported');
                return;
            }

            try {
                wakeLock = await navigator.wakeLock.request('screen');
                console.log('Wake Lock acquired');

                // Show indicator
                if (this.wakeLockIndicator) {
                    this.wakeLockIndicator.style.display = 'flex';
                }

                // Handle wake lock release (e.g., tab visibility change)
                wakeLock.addEventListener('release', () => {
                    console.log('Wake Lock released');
                    if (this.wakeLockIndicator) {
                        this.wakeLockIndicator.style.display = 'none';
                    }
                });
            } catch (err) {
                // AC6: Handle errors gracefully (permission denied, battery saver, etc.)
                console.error('Failed to acquire Wake Lock:', err);
            }

            // Re-request wake lock when page becomes visible again
            // This handles cases where browser releases wake lock on tab switch
            this.handleVisibilityChange = async () => {
                if (wakeLock !== null && document.visibilityState === 'visible') {
                    try {
                        wakeLock = await navigator.wakeLock.request('screen');
                        console.log('Wake Lock re-acquired after visibility change');
                        if (this.wakeLockIndicator) {
                            this.wakeLockIndicator.style.display = 'flex';
                        }
                    } catch (err) {
                        console.error('Failed to re-acquire Wake Lock:', err);
                    }
                }
            };

            document.addEventListener('visibilitychange', this.handleVisibilityChange);
        }

        // AC6: Release wake lock
        /**
         * Releases the Wake Lock and removes visibility change listener
         */
        releaseWakeLock() {
            if (wakeLock) {
                wakeLock.release()
                    .then(() => {
                        wakeLock = null;
                        if (this.wakeLockIndicator) {
                            this.wakeLockIndicator.style.display = 'none';
                        }
                    })
                    .catch(err => console.error('Failed to release Wake Lock:', err));
            }

            // Remove visibility change listener
            if (this.handleVisibilityChange) {
                document.removeEventListener('visibilitychange', this.handleVisibilityChange);
                this.handleVisibilityChange = null;
            }
        }

        // AC5: Initialize step-by-step navigation
        /**
         * Initializes step-by-step navigation for instructions
         * - Hides all instructions except the first
         * - Creates navigation controls (Previous, Indicator, Next)
         * - Adds keyboard navigation support
         */
        initStepByStepNavigation() {
            const instructions = document.querySelectorAll('ol li');
            if (instructions.length === 0) return;

            this.currentStep = 0;
            this.instructions = instructions;

            // Hide all instructions except the first
            instructions.forEach((instruction, index) => {
                instruction.style.display = index === 0 ? 'flex' : 'none';
            });

            // Create navigation controls
            this.createStepNavigationControls();

            // Add keyboard navigation
            this.stepKeydownHandler = (e) => this.handleStepKeydown(e);
            document.addEventListener('keydown', this.stepKeydownHandler);

            // Update step display
            this.updateStep();
        }

        /**
         * Destroys step-by-step navigation
         * - Shows all instructions
         * - Removes navigation controls
         * - Removes keyboard listener
         */
        destroyStepByStepNavigation() {
            if (!this.instructions) return;

            // Show all instructions
            this.instructions.forEach(instruction => {
                instruction.style.display = 'flex';
            });

            // Remove navigation controls
            if (this.navContainer) {
                this.navContainer.remove();
                this.navContainer = null;
            }

            // Remove keyboard listener
            if (this.stepKeydownHandler) {
                document.removeEventListener('keydown', this.stepKeydownHandler);
                this.stepKeydownHandler = null;
            }

            this.instructions = null;
            this.currentStep = 0;
        }

        // Create step navigation controls (Previous, Indicator, Next)
        createStepNavigationControls() {
            const navContainer = document.createElement('div');
            navContainer.className = 'flex gap-4 mt-8 justify-center items-center flex-wrap';

            // Previous button
            const prevButton = document.createElement('button');
            prevButton.setAttribute('data-testid', 'step-previous');
            prevButton.setAttribute('aria-label', 'Previous step');
            prevButton.className = 'px-8 py-4 bg-blue-600 text-white rounded-lg text-xl font-bold touch-target hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed';
            prevButton.textContent = '← Previous';
            prevButton.addEventListener('click', () => this.previousStep());

            // Step indicator
            const stepIndicator = document.createElement('div');
            stepIndicator.setAttribute('data-testid', 'step-indicator');
            stepIndicator.setAttribute('aria-live', 'polite');
            stepIndicator.className = 'px-6 py-4 bg-gray-200 rounded-lg text-xl font-bold text-gray-900';

            // Next button
            const nextButton = document.createElement('button');
            nextButton.setAttribute('data-testid', 'step-next');
            nextButton.setAttribute('aria-label', 'Next step');
            nextButton.className = 'px-8 py-4 bg-blue-600 text-white rounded-lg text-xl font-bold touch-target hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed';
            nextButton.textContent = 'Next →';
            nextButton.addEventListener('click', () => this.nextStep());

            navContainer.appendChild(prevButton);
            navContainer.appendChild(stepIndicator);
            navContainer.appendChild(nextButton);

            // Insert navigation after instructions list
            const instructionsSection = document.querySelector('ol');
            if (instructionsSection) {
                instructionsSection.parentNode.insertBefore(navContainer, instructionsSection.nextSibling);
            }

            this.navContainer = navContainer;
            this.prevButton = prevButton;
            this.stepIndicator = stepIndicator;
            this.nextButton = nextButton;
        }

        /**
         * Navigates to the previous instruction step
         */
        previousStep() {
            if (this.currentStep > 0) {
                this.currentStep--;
                this.updateStep();
            }
        }

        /**
         * Navigates to the next instruction step
         */
        nextStep() {
            if (this.currentStep < this.instructions.length - 1) {
                this.currentStep++;
                this.updateStep();
            }
        }

        /**
         * Updates the step display
         * - Shows current instruction
         * - Updates step indicator text
         * - Updates button states (enabled/disabled)
         */
        updateStep() {
            if (!this.instructions) return;

            // Hide all instructions
            this.instructions.forEach(inst => inst.style.display = 'none');

            // Show current instruction
            this.instructions[this.currentStep].style.display = 'flex';

            // Update step indicator
            if (this.stepIndicator) {
                this.stepIndicator.textContent = `Step ${this.currentStep + 1} of ${this.instructions.length}`;
            }

            // Update button states
            if (this.prevButton) {
                this.prevButton.disabled = this.currentStep === 0;
            }

            if (this.nextButton) {
                this.nextButton.disabled = this.currentStep === this.instructions.length - 1;
                this.nextButton.textContent = this.currentStep === this.instructions.length - 1 ? '✓ Done' : 'Next →';
            }
        }

        // AC5: Keyboard navigation (arrow keys, spacebar)
        /**
         * Handles keyboard navigation for step-by-step mode
         * @param {KeyboardEvent} e - The keyboard event
         */
        handleStepKeydown(e) {
            // ArrowLeft or ArrowUp = Previous
            if ((e.key === 'ArrowLeft' || e.key === 'ArrowUp') && this.currentStep > 0) {
                this.previousStep();
                e.preventDefault();
            }
            // ArrowRight, ArrowDown, or Space = Next
            else if ((e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') && this.currentStep < this.instructions.length - 1) {
                this.nextStep();
                e.preventDefault();
            }
        }
    }

    // Initialize Kitchen Mode Manager on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            new KitchenModeManager();
        });
    } else {
        new KitchenModeManager();
    }
})();
