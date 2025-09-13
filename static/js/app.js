/*
 * ImKitchen Application JavaScript
 */

document.addEventListener('DOMContentLoaded', function() {
    console.log('ImKitchen app initialized');
    
    // Initialize application features
    initializeFormHandling();
    initializeNavigation();
});

function initializeFormHandling() {
    // Add form validation and enhancement
    const forms = document.querySelectorAll('form');
    forms.forEach(form => {
        form.addEventListener('submit', function(e) {
            if (!validateForm(form)) {
                e.preventDefault();
            }
        });
    });
}

function validateForm(form) {
    let isValid = true;
    const requiredFields = form.querySelectorAll('[required]');
    
    requiredFields.forEach(field => {
        if (!field.value.trim()) {
            showFieldError(field, 'This field is required');
            isValid = false;
        } else {
            clearFieldError(field);
        }
    });
    
    return isValid;
}

function showFieldError(field, message) {
    clearFieldError(field);
    
    const errorDiv = document.createElement('div');
    errorDiv.className = 'form-error';
    errorDiv.textContent = message;
    
    field.parentNode.classList.add('has-error');
    field.parentNode.appendChild(errorDiv);
}

function clearFieldError(field) {
    const parent = field.parentNode;
    const existingError = parent.querySelector('.form-error');
    if (existingError) {
        existingError.remove();
    }
    parent.classList.remove('has-error');
}

function initializeNavigation() {
    // Add active state to current navigation item
    const currentPath = window.location.pathname;
    const navLinks = document.querySelectorAll('.nav-menu a');
    
    navLinks.forEach(link => {
        if (link.getAttribute('href') === currentPath) {
            link.style.backgroundColor = '#f0f0f0';
            link.style.color = '#2c5aa0';
        }
    });
}

// Utility functions
window.ImKitchen = {
    showNotification: function(message, type = 'info') {
        // Simple notification system
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.textContent = message;
        
        document.body.appendChild(notification);
        
        // Auto-remove after 3 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 3000);
    },
    
    formatTime: function(minutes) {
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        
        if (hours > 0) {
            return `${hours}h ${mins}m`;
        }
        return `${mins}m`;
    }
};