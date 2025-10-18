#!/bin/bash
# Story 4.6: Quick Testing Script for Notifications

set -e

echo "🧪 Story 4.6 - Notification System Test Script"
echo "=============================================="
echo ""

# Check if database exists
if [ ! -f "imkitchen.db" ]; then
    echo "❌ Error: imkitchen.db not found. Please run migrations first."
    exit 1
fi

echo "📊 Current Notification Status"
echo "------------------------------"

# Show pending notifications
echo "Pending notifications:"
sqlite3 imkitchen.db <<SQL
SELECT
    id,
    user_id,
    recipe_id,
    meal_date,
    scheduled_time,
    reminder_type,
    prep_hours,
    status
FROM notifications
WHERE status = 'pending'
ORDER BY scheduled_time;
SQL

echo ""
echo "Push subscriptions:"
sqlite3 imkitchen.db <<SQL
SELECT
    id,
    user_id,
    endpoint,
    created_at
FROM push_subscriptions;
SQL

echo ""
echo "📈 Notification Statistics"
echo "-------------------------"
sqlite3 imkitchen.db <<SQL
SELECT
    status,
    COUNT(*) as count
FROM notifications
GROUP BY status;
SQL

echo ""
echo "✅ Test Commands Available:"
echo ""
echo "# View all notifications (requires login token):"
echo 'curl -H "Cookie: session=YOUR_TOKEN" http://localhost:3000/api/notifications'
echo ""
echo "# Test dismiss (replace IDs):"
echo 'curl -X POST -H "Cookie: session=YOUR_TOKEN" http://localhost:3000/api/notifications/{id}/dismiss'
echo ""
echo "# Test snooze for 2 hours:"
echo 'curl -X POST -H "Cookie: session=YOUR_TOKEN" -d "duration_hours=2" http://localhost:3000/api/notifications/{id}/snooze'
echo ""
echo "# Subscribe to push notifications:"
echo 'curl -X POST -H "Cookie: session=YOUR_TOKEN" -H "Content-Type: application/json" -d @push-subscription.json http://localhost:3000/api/notifications/subscribe'
echo ""
echo "📖 For detailed testing guide, see: docs/testing/story-4.6-testing-guide.md"
