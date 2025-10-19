/**
 * Polyfills for Cross-Browser Compatibility
 *
 * Story 5.7: Cross-Browser Compatibility
 * AC-4: Polyfills for missing features where feasible
 * AC-8: JavaScript compatibility via transpilation (ES2015+ support)
 *
 * Conditionally loads polyfills for older browsers.
 * Target browsers: iOS Safari 14+, Chrome 90+, Firefox 88+
 *
 * Note: Modern target browsers support most ES2015+ features natively.
 * These polyfills are fallbacks for edge cases and older versions.
 */

(function () {
  'use strict';

  // Check if polyfills are needed
  const needsPolyfills = !window.Promise ||
                          !window.fetch ||
                          !window.IntersectionObserver ||
                          !Object.assign;

  if (!needsPolyfills) {
    console.info('No polyfills needed - browser supports modern features');
    return;
  }

  console.warn('Loading polyfills for legacy browser support');

  // Promise polyfill (for very old browsers)
  if (!window.Promise) {
    console.warn('Promise API not supported - adding polyfill');

    // Minimal Promise polyfill (production apps should use core-js)
    window.Promise = function (executor) {
      const callbacks = [];
      let value;
      let resolved = false;

      function resolve(val) {
        if (resolved) return;
        resolved = true;
        value = val;
        callbacks.forEach(function (cb) {
          setTimeout(function () {
            cb(value);
          }, 0);
        });
      }

      this.then = function (onFulfilled) {
        return new Promise(function (innerResolve) {
          function handleResolved() {
            try {
              const result = onFulfilled ? onFulfilled(value) : value;
              innerResolve(result);
            } catch (e) {
              console.error('Promise error:', e);
            }
          }

          if (resolved) {
            setTimeout(handleResolved, 0);
          } else {
            callbacks.push(handleResolved);
          }
        });
      };

      this.catch = function (onRejected) {
        return this.then(null, onRejected);
      };

      try {
        executor(resolve, function reject(reason) {
          console.error('Promise rejected:', reason);
        });
      } catch (e) {
        console.error('Promise executor error:', e);
      }
    };

    Promise.resolve = function (value) {
      return new Promise(function (resolve) {
        resolve(value);
      });
    };

    Promise.reject = function (reason) {
      return new Promise(function (resolve, reject) {
        reject(reason);
      });
    };
  }

  // Fetch API polyfill (for older browsers)
  if (!window.fetch) {
    console.warn('Fetch API not supported - adding XMLHttpRequest fallback');

    window.fetch = function (url, options) {
      options = options || {};

      return new Promise(function (resolve, reject) {
        const xhr = new XMLHttpRequest();
        xhr.open(options.method || 'GET', url);

        // Set headers
        if (options.headers) {
          Object.keys(options.headers).forEach(function (key) {
            xhr.setRequestHeader(key, options.headers[key]);
          });
        }

        xhr.onload = function () {
          resolve({
            ok: xhr.status >= 200 && xhr.status < 300,
            status: xhr.status,
            statusText: xhr.statusText,
            text: function () {
              return Promise.resolve(xhr.responseText);
            },
            json: function () {
              return Promise.resolve(JSON.parse(xhr.responseText));
            },
            headers: {
              get: function (name) {
                return xhr.getResponseHeader(name);
              },
            },
          });
        };

        xhr.onerror = function () {
          reject(new TypeError('Network request failed'));
        };

        xhr.ontimeout = function () {
          reject(new TypeError('Network request timed out'));
        };

        xhr.send(options.body || null);
      });
    };
  }

  // IntersectionObserver polyfill (for lazy-loading images)
  if (!window.IntersectionObserver) {
    console.warn('IntersectionObserver not supported - adding fallback');

    // Simplified IntersectionObserver polyfill
    window.IntersectionObserver = function (callback) {
      this.observe = function (target) {
        // Immediately consider all targets visible (conservative fallback)
        setTimeout(function () {
          callback([
            {
              isIntersecting: true,
              target: target,
              intersectionRatio: 1,
            },
          ]);
        }, 100);
      };

      this.unobserve = function () {};
      this.disconnect = function () {};
    };
  }

  // Object.assign polyfill (for older browsers)
  if (!Object.assign) {
    console.warn('Object.assign not supported - adding polyfill');

    Object.assign = function (target) {
      if (target == null) {
        throw new TypeError('Cannot convert undefined or null to object');
      }

      const to = Object(target);

      for (let index = 1; index < arguments.length; index++) {
        const nextSource = arguments[index];

        if (nextSource != null) {
          for (const key in nextSource) {
            if (Object.prototype.hasOwnProperty.call(nextSource, key)) {
              to[key] = nextSource[key];
            }
          }
        }
      }

      return to;
    };
  }

  // Array.from polyfill
  if (!Array.from) {
    Array.from = function (arrayLike) {
      return Array.prototype.slice.call(arrayLike);
    };
  }

  // Array.prototype.find polyfill
  if (!Array.prototype.find) {
    Array.prototype.find = function (predicate) {
      for (let i = 0; i < this.length; i++) {
        if (predicate(this[i], i, this)) {
          return this[i];
        }
      }
      return undefined;
    };
  }

  // String.prototype.includes polyfill
  if (!String.prototype.includes) {
    String.prototype.includes = function (search) {
      return this.indexOf(search) !== -1;
    };
  }

  // String.prototype.startsWith polyfill
  if (!String.prototype.startsWith) {
    String.prototype.startsWith = function (search) {
      return this.indexOf(search) === 0;
    };
  }

  // String.prototype.endsWith polyfill
  if (!String.prototype.endsWith) {
    String.prototype.endsWith = function (search) {
      return this.indexOf(search, this.length - search.length) !== -1;
    };
  }

  // Number.isNaN polyfill
  if (!Number.isNaN) {
    Number.isNaN = function (value) {
      return typeof value === 'number' && isNaN(value);
    };
  }

  console.info('Polyfills loaded successfully');
})();
