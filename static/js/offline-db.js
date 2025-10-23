/**
 * IndexedDB wrapper utilities for offline data persistence
 * Story 5.3 - Offline Recipe Access
 *
 * Database: imkitchen-offline
 * Object Stores:
 *   - recipes: { id, title, ingredients, instructions, image_url, cached_at }
 *   - meal_plans: { id, user_id, start_date, meals[], cached_at }
 *   - shopping_lists: { id, week_start_date, items[], cached_at }
 *   - sync_queue: { request_id, url, method, body, retry_count, queued_at }
 */

const DB_NAME = 'imkitchen-offline';
const DB_VERSION = 1;

/**
 * Open IndexedDB connection
 * @returns {Promise<IDBDatabase>}
 */
function openDatabase() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => {
      console.error('Failed to open IndexedDB:', request.error);
      reject(request.error);
    };

    request.onsuccess = () => {
      console.log('IndexedDB opened successfully');
      resolve(request.result);
    };

    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      console.log('Upgrading IndexedDB schema to version', DB_VERSION);

      // Create recipes store
      if (!db.objectStoreNames.contains('recipes')) {
        const recipesStore = db.createObjectStore('recipes', { keyPath: 'id' });
        recipesStore.createIndex('cached_at', 'cached_at', { unique: false });
        recipesStore.createIndex('title', 'title', { unique: false });
      }

      // Create meal_plans store
      if (!db.objectStoreNames.contains('meal_plans')) {
        const mealPlansStore = db.createObjectStore('meal_plans', { keyPath: 'id' });
        mealPlansStore.createIndex('user_id', 'user_id', { unique: false });
        mealPlansStore.createIndex('start_date', 'start_date', { unique: false });
        mealPlansStore.createIndex('cached_at', 'cached_at', { unique: false });
      }

      // Create shopping_lists store
      if (!db.objectStoreNames.contains('shopping_lists')) {
        const shoppingListsStore = db.createObjectStore('shopping_lists', { keyPath: 'id' });
        shoppingListsStore.createIndex('week_start_date', 'week_start_date', { unique: false });
        shoppingListsStore.createIndex('cached_at', 'cached_at', { unique: false });
      }

      // Create sync_queue store
      if (!db.objectStoreNames.contains('sync_queue')) {
        const syncQueueStore = db.createObjectStore('sync_queue', { keyPath: 'request_id', autoIncrement: true });
        syncQueueStore.createIndex('queued_at', 'queued_at', { unique: false });
        syncQueueStore.createIndex('retry_count', 'retry_count', { unique: false });
      }
    };
  });
}

/**
 * Generic get operation
 * @param {string} storeName - Name of the object store
 * @param {string|number} key - Primary key
 * @returns {Promise<object|null>}
 */
async function get(storeName, key) {
  const db = await openDatabase();
  try {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.get(key);

      request.onsuccess = () => {
        resolve(request.result || null);
      };

      request.onerror = () => {
        console.error(`Failed to get ${storeName}[${key}]:`, request.error);
        reject(request.error);
      };

      transaction.oncomplete = () => db.close();
      transaction.onerror = () => {
        db.close();
        reject(transaction.error);
      };
      transaction.onabort = () => {
        db.close();
        reject(new Error('Transaction aborted'));
      };
    });
  } catch (error) {
    db.close();
    throw error;
  }
}

/**
 * Generic put operation (insert or update)
 * @param {string} storeName - Name of the object store
 * @param {object} data - Data to store (must include key)
 * @returns {Promise<void>}
 */
async function put(storeName, data) {
  const db = await openDatabase();
  try {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.put(data);

      request.onsuccess = () => {
        console.log(`Stored ${storeName}[${request.result}]`);
        resolve();
      };

      request.onerror = () => {
        const error = request.error;

        // Handle quota exceeded error specifically
        if (error && error.name === 'QuotaExceededError') {
          console.error(`Storage quota exceeded for ${storeName}`);
          showStorageQuotaWarning();
        } else {
          console.error(`Failed to put ${storeName}:`, error);
        }

        reject(error);
      };

      transaction.oncomplete = () => db.close();
      transaction.onerror = () => {
        db.close();
        reject(transaction.error);
      };
      transaction.onabort = () => {
        db.close();
        reject(new Error('Transaction aborted'));
      };
    });
  } catch (error) {
    db.close();

    // Handle quota exceeded at catch level as well
    if (error && error.name === 'QuotaExceededError') {
      console.error('Storage quota exceeded');
      showStorageQuotaWarning();
    }

    throw error;
  }
}

/**
 * Generic delete operation
 * @param {string} storeName - Name of the object store
 * @param {string|number} key - Primary key
 * @returns {Promise<void>}
 */
async function remove(storeName, key) {
  const db = await openDatabase();
  try {
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.delete(key);

      request.onsuccess = () => {
        console.log(`Deleted ${storeName}[${key}]`);
        resolve();
      };

      request.onerror = () => {
        console.error(`Failed to delete ${storeName}[${key}]:`, request.error);
        reject(request.error);
      };

      transaction.oncomplete = () => db.close();
      transaction.onerror = () => {
        db.close();
        reject(transaction.error);
      };
      transaction.onabort = () => {
        db.close();
        reject(new Error('Transaction aborted'));
      };
    });
  } catch (error) {
    db.close();
    throw error;
  }
}

/**
 * Get all records from a store
 * @param {string} storeName - Name of the object store
 * @returns {Promise<Array<object>>}
 */
async function getAll(storeName) {
  const db = await openDatabase();
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readonly');
    const store = transaction.objectStore(storeName);
    const request = store.getAll();

    request.onsuccess = () => {
      resolve(request.result || []);
    };

    request.onerror = () => {
      console.error(`Failed to get all from ${storeName}:`, request.error);
      reject(request.error);
    };

    transaction.oncomplete = () => db.close();
  });
}

/**
 * Clear all records from a store
 * @param {string} storeName - Name of the object store
 * @returns {Promise<void>}
 */
async function clear(storeName) {
  const db = await openDatabase();
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readwrite');
    const store = transaction.objectStore(storeName);
    const request = store.clear();

    request.onsuccess = () => {
      console.log(`Cleared ${storeName}`);
      resolve();
    };

    request.onerror = () => {
      console.error(`Failed to clear ${storeName}:`, request.error);
      reject(request.error);
    };

    transaction.oncomplete = () => db.close();
  });
}

// ============================================================
// Recipe-specific operations
// ============================================================

/**
 * Cache a recipe for offline access
 * @param {object} recipe - Recipe data
 * @returns {Promise<void>}
 */
async function cacheRecipe(recipe) {
  const recipeData = {
    ...recipe,
    cached_at: new Date().toISOString()
  };
  return put('recipes', recipeData);
}

/**
 * Get cached recipe by ID
 * @param {string} recipeId
 * @returns {Promise<object|null>}
 */
async function getCachedRecipe(recipeId) {
  return get('recipes', recipeId);
}

/**
 * Get all cached recipes
 * @returns {Promise<Array<object>>}
 */
async function getAllCachedRecipes() {
  return getAll('recipes');
}

// ============================================================
// Meal plan-specific operations
// ============================================================

/**
 * Cache a meal plan for offline access
 * @param {object} mealPlan - Meal plan data
 * @returns {Promise<void>}
 */
async function cacheMealPlan(mealPlan) {
  const mealPlanData = {
    ...mealPlan,
    cached_at: new Date().toISOString()
  };
  return put('meal_plans', mealPlanData);
}

/**
 * Get cached meal plan by ID
 * @param {string} mealPlanId
 * @returns {Promise<object|null>}
 */
async function getCachedMealPlan(mealPlanId) {
  return get('meal_plans', mealPlanId);
}

/**
 * Get active meal plan (most recent)
 * @returns {Promise<object|null>}
 */
async function getActiveMealPlan() {
  const mealPlans = await getAll('meal_plans');
  if (mealPlans.length === 0) return null;

  // Sort by start_date descending, return most recent
  mealPlans.sort((a, b) => new Date(b.start_date) - new Date(a.start_date));
  return mealPlans[0];
}

// ============================================================
// Shopping list-specific operations
// ============================================================

/**
 * Cache a shopping list for offline access
 * @param {object} shoppingList - Shopping list data
 * @returns {Promise<void>}
 */
async function cacheShoppingList(shoppingList) {
  const shoppingListData = {
    ...shoppingList,
    cached_at: new Date().toISOString()
  };
  return put('shopping_lists', shoppingListData);
}

/**
 * Get cached shopping list by ID
 * @param {string} shoppingListId
 * @returns {Promise<object|null>}
 */
async function getCachedShoppingList(shoppingListId) {
  return get('shopping_lists', shoppingListId);
}

/**
 * Get current week's shopping list (most recent)
 * @returns {Promise<object|null>}
 */
async function getCurrentShoppingList() {
  const shoppingLists = await getAll('shopping_lists');
  if (shoppingLists.length === 0) return null;

  // Sort by week_start_date descending, return most recent
  shoppingLists.sort((a, b) => new Date(b.week_start_date) - new Date(a.start_date));
  return shoppingLists[0];
}

// ============================================================
// Sync queue operations
// ============================================================

/**
 * Add a request to the sync queue
 * @param {object} requestData - { url, method, body, headers }
 * @returns {Promise<number>} Request ID
 */
async function queueRequest(requestData) {
  const db = await openDatabase();
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(['sync_queue'], 'readwrite');
    const store = transaction.objectStore('sync_queue');

    const queuedRequest = {
      ...requestData,
      retry_count: 0,
      queued_at: new Date().toISOString()
    };

    const request = store.add(queuedRequest);

    request.onsuccess = () => {
      console.log('Request queued:', request.result);
      resolve(request.result);
    };

    request.onerror = () => {
      console.error('Failed to queue request:', request.error);
      reject(request.error);
    };

    transaction.oncomplete = () => db.close();
  });
}

/**
 * Get all queued requests
 * @returns {Promise<Array<object>>}
 */
async function getQueuedRequests() {
  return getAll('sync_queue');
}

/**
 * Remove a request from the sync queue
 * @param {number} requestId
 * @returns {Promise<void>}
 */
async function removeQueuedRequest(requestId) {
  return remove('sync_queue', requestId);
}

/**
 * Clear the entire sync queue
 * @returns {Promise<void>}
 */
async function clearSyncQueue() {
  return clear('sync_queue');
}

// ============================================================
// Cache management utilities
// ============================================================

/**
 * Get cache statistics
 * @returns {Promise<object>} { recipes: number, mealPlans: number, shoppingLists: number, queuedRequests: number }
 */
async function getCacheStats() {
  const [recipes, mealPlans, shoppingLists, queuedRequests] = await Promise.all([
    getAll('recipes'),
    getAll('meal_plans'),
    getAll('shopping_lists'),
    getAll('sync_queue')
  ]);

  return {
    recipes: recipes.length,
    mealPlans: mealPlans.length,
    shoppingLists: shoppingLists.length,
    queuedRequests: queuedRequests.length,
    totalCached: recipes.length + mealPlans.length + shoppingLists.length
  };
}

/**
 * Clear all cached data (except sync queue)
 * @returns {Promise<void>}
 */
async function clearAllCache() {
  await Promise.all([
    clear('recipes'),
    clear('meal_plans'),
    clear('shopping_lists')
  ]);
  console.log('All cached data cleared');
}

// ============================================================
// Storage quota management
// ============================================================

/**
 * Show storage quota warning to user
 * Creates a dismissible notification when storage quota is exceeded
 */
function showStorageQuotaWarning() {
  // Check if warning already displayed
  if (document.getElementById('storage-quota-warning')) {
    return;
  }

  const warning = document.createElement('div');
  warning.id = 'storage-quota-warning';
  warning.className = 'fixed bottom-4 right-4 bg-yellow-50 border border-yellow-200 text-yellow-900 p-4 rounded-lg shadow-lg max-w-md z-50';
  warning.innerHTML = `
    <div class="flex items-start">
      <svg class="w-5 h-5 mr-2 text-yellow-600 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path>
      </svg>
      <div class="flex-1">
        <h3 class="font-medium">Storage Almost Full</h3>
        <p class="text-sm mt-1">Your device storage is running low. Some offline data may not be saved. Consider clearing old cached recipes.</p>
        <button onclick="this.closest('#storage-quota-warning').remove()" class="mt-2 text-sm font-medium text-yellow-700 underline">Dismiss</button>
      </div>
    </div>
  `;

  document.body.appendChild(warning);

  // Auto-dismiss after 10 seconds
  setTimeout(() => {
    if (warning.parentNode) {
      warning.remove();
    }
  }, 10000);
}

/**
 * Export offline-db utilities as a global object for non-module scripts
 * Supports both window context (main thread) and self context (service worker)
 */
const offlineDB = {
  openDatabase,
  get,
  put,
  remove,
  getAll,
  clear,
  cacheRecipe,
  getCachedRecipe,
  getAllCachedRecipes,
  cacheMealPlan,
  getCachedMealPlan,
  getActiveMealPlan,
  cacheShoppingList,
  getCachedShoppingList,
  getCurrentShoppingList,
  queueRequest,
  getQueuedRequests,
  removeQueuedRequest,
  clearSyncQueue,
  getCacheStats,
  clearAllCache
};

// Export to window (main thread) or self (service worker)
if (typeof window !== 'undefined') {
  window.offlineDB = offlineDB;
}
if (typeof self !== 'undefined') {
  self.offlineDB = offlineDB;
}
