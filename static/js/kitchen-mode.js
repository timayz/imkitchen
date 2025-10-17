// Kitchen Mode Progressive Disclosure
// Story 3.5 AC-7: Progressive disclosure for instructions in kitchen mode
// Action Item 2: Extracted from inline script for CSP compliance

(function() {
    'use strict';

    // Only initialize if kitchen mode is active
    const isKitchenMode = document.body.classList.contains('kitchen-mode');
    if (!isKitchenMode) return;

    const instructions = document.querySelectorAll('ol li');
    if (instructions.length === 0) return;

    let currentStep = 0;

    // Hide all instructions initially except the first
    instructions.forEach((instruction, index) => {
        if (index !== 0) {
            instruction.style.display = 'none';
        }
    });

    // Create navigation buttons
    const navContainer = document.createElement('div');
    navContainer.style.cssText = 'display: flex; gap: 1rem; margin-top: 2rem; justify-content: center;';

    const prevButton = document.createElement('button');
    prevButton.textContent = '← Previous';
    prevButton.style.cssText = 'padding: 1rem 2rem; background-color: #3182ce; color: white; border-radius: 0.5rem; font-size: 1.25rem; font-weight: bold; cursor: pointer; border: none;';
    prevButton.disabled = true;
    prevButton.style.opacity = '0.5';

    const stepIndicator = document.createElement('div');
    stepIndicator.style.cssText = 'padding: 1rem 2rem; background-color: #edf2f7; border-radius: 0.5rem; font-size: 1.25rem; font-weight: bold; display: flex; align-items: center;';
    stepIndicator.textContent = `Step 1 of ${instructions.length}`;

    const nextButton = document.createElement('button');
    nextButton.textContent = 'Next →';
    nextButton.style.cssText = 'padding: 1rem 2rem; background-color: #3182ce; color: white; border-radius: 0.5rem; font-size: 1.25rem; font-weight: bold; cursor: pointer; border: none;';

    navContainer.appendChild(prevButton);
    navContainer.appendChild(stepIndicator);
    navContainer.appendChild(nextButton);

    // Insert navigation after the instructions list
    const instructionsSection = document.querySelector('ol');
    if (instructionsSection) {
        instructionsSection.parentNode.insertBefore(navContainer, instructionsSection.nextSibling);
    }

    function updateStep() {
        // Hide all instructions
        instructions.forEach(inst => inst.style.display = 'none');

        // Show current instruction
        instructions[currentStep].style.display = 'flex';

        // Update step indicator
        stepIndicator.textContent = `Step ${currentStep + 1} of ${instructions.length}`;

        // Update button states
        prevButton.disabled = currentStep === 0;
        prevButton.style.opacity = currentStep === 0 ? '0.5' : '1';
        prevButton.style.cursor = currentStep === 0 ? 'not-allowed' : 'pointer';

        nextButton.disabled = currentStep === instructions.length - 1;
        nextButton.style.opacity = currentStep === instructions.length - 1 ? '0.5' : '1';
        nextButton.style.cursor = currentStep === instructions.length - 1 ? 'not-allowed' : 'pointer';

        if (currentStep === instructions.length - 1) {
            nextButton.textContent = '✓ Done';
        } else {
            nextButton.textContent = 'Next →';
        }
    }

    // Action Item 4: Keyboard navigation support
    function handleKeydown(e) {
        if (e.key === 'ArrowLeft' && currentStep > 0) {
            currentStep--;
            updateStep();
            e.preventDefault();
        } else if (e.key === 'ArrowRight' && currentStep < instructions.length - 1) {
            currentStep++;
            updateStep();
            e.preventDefault();
        }
    }

    prevButton.addEventListener('click', function() {
        if (currentStep > 0) {
            currentStep--;
            updateStep();
        }
    });

    nextButton.addEventListener('click', function() {
        if (currentStep < instructions.length - 1) {
            currentStep++;
            updateStep();
        }
    });

    // Action Item 4: Add keyboard navigation
    document.addEventListener('keydown', handleKeydown);

    // Initialize
    updateStep();
})();
