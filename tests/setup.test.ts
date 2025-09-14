/**
 * Basic test setup verification
 * This test ensures Jest and testing environment are properly configured
 */

describe('Test Environment Setup', () => {
  it('should have Jest configured correctly', () => {
    expect(true).toBe(true);
  });

  it('should have access to DOM environment', () => {
    const element = document.createElement('div');
    element.textContent = 'Test';
    expect(element.textContent).toBe('Test');
  });

  it('should have localStorage mock available', () => {
    localStorage.setItem('test', 'value');
    expect(localStorage.getItem('test')).toBe('value');
  });
});
