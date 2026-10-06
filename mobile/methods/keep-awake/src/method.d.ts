/** Keep the screen on while cooking (FLAG_KEEP_SCREEN_ON / isIdleTimerDisabled). */
export interface SetEnabledRequest {
  enabled: boolean;
}

export interface SetEnabledResponse {
  /**
   * @default true
   */
  success: boolean;
}

declare function setEnabled(params: SetEnabledRequest, callback: (result: SetEnabledResponse) => void): void;
