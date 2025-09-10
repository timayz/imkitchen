import '@testing-library/jest-native/extend-expect';

// Mock React Native components
jest.mock('react-native', () => {
  const RN = jest.requireActual('react-native');
  
  RN.NativeModules.SettingsManager = {
    settings: {},
  };
  
  return RN;
});

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () => ({
  getItem: jest.fn(() => Promise.resolve(null)),
  setItem: jest.fn(() => Promise.resolve()),
  removeItem: jest.fn(() => Promise.resolve()),
  multiRemove: jest.fn(() => Promise.resolve()),
  getAllKeys: jest.fn(() => Promise.resolve([])),
  multiGet: jest.fn(() => Promise.resolve([])),
  multiSet: jest.fn(() => Promise.resolve()),
}));

// Mock Slider component
jest.mock('@react-native-community/slider', () => {
  const React = require('react');
  const { View } = require('react-native');
  
  return React.forwardRef((props: any, ref: any) => {
    return React.createElement(View, {
      ...props,
      ref,
      testID: 'slider',
      onValueChange: props.onValueChange,
      onSlidingComplete: props.onSlidingComplete,
    });
  });
});

// Mock console methods for cleaner test output
global.console = {
  ...console,
  // Uncomment to ignore console logs and warnings in tests
  // log: jest.fn(),
  // warn: jest.fn(),
  error: jest.fn(),
};

// Setup test environment
beforeEach(() => {
  jest.clearAllMocks();
});

afterEach(() => {
  jest.clearAllTimers();
});