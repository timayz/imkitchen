# Stripe Setup Guide

This guide walks you through setting up Stripe for the imkitchen application's premium subscription feature.

## Prerequisites

- A Stripe account (create one at https://stripe.com)
- Access to the Stripe Dashboard

## 1. Create Stripe Account and Get Test Keys

1. Sign up for a Stripe account at https://stripe.com or log in to your existing account
2. Navigate to the Stripe Dashboard
3. Toggle to **Test Mode** in the top-right corner (you should see a "Test Mode" indicator)
4. Go to **Developers > API keys**
5. Copy your **Publishable key** (starts with `pk_test_`)
6. Copy your **Secret key** (starts with `sk_test_`)
   - Click "Reveal test key" to view the secret key
   - **Important**: Keep your secret key secure and never commit it to version control

## 2. Create Price Object for $9.99/month Subscription

1. In the Stripe Dashboard, go to **Products**
2. Click **+ Add product**
3. Enter the product details:
   - **Name**: `Premium Subscription` (or your preferred name)
   - **Description**: `Premium imkitchen subscription with unlimited recipes`
4. Under **Pricing**:
   - Select **Recurring** pricing model
   - Set **Price**: `9.99` USD
   - Set **Billing period**: `Monthly`
5. Click **Save product**
6. Copy the **Price ID** (starts with `price_`) from the pricing section
   - You'll need this for the `STRIPE_PRICE_ID` environment variable

## 3. Configure Webhook Endpoint and Secret

Webhooks allow Stripe to notify your application when events occur (e.g., successful payment, subscription canceled).

### 3.1 Set Up Local Webhook Testing with Stripe CLI (Development)

For local development, use the Stripe CLI to forward webhook events:

```bash
# Install Stripe CLI (https://stripe.com/docs/stripe-cli)
# macOS
brew install stripe/stripe-cli/stripe

# Linux
wget https://github.com/stripe/stripe-cli/releases/download/v1.19.4/stripe_1.19.4_linux_x86_64.tar.gz
tar -xvf stripe_1.19.4_linux_x86_64.tar.gz
sudo mv stripe /usr/local/bin

# Login to Stripe
stripe login

# Forward webhook events to your local server
stripe listen --forward-to localhost:3000/webhooks/stripe
```

The Stripe CLI will output a webhook signing secret (starts with `whsec_`). Use this for `STRIPE_WEBHOOK_SECRET`.

### 3.2 Set Up Production Webhook Endpoint

For production deployments:

1. In the Stripe Dashboard, go to **Developers > Webhooks**
2. Click **+ Add endpoint**
3. Enter your endpoint URL: `https://yourdomain.com/webhooks/stripe`
4. Select events to listen for:
   - `checkout.session.completed` - When a customer completes checkout
   - `customer.subscription.updated` - When subscription status changes
   - `customer.subscription.deleted` - When subscription is canceled
5. Click **Add endpoint**
6. Copy the **Signing secret** (starts with `whsec_`)
   - You'll need this for the `STRIPE_WEBHOOK_SECRET` environment variable

## 4. Set Environment Variables

Add the following environment variables to your `.env` file:

```env
# Stripe Configuration
STRIPE_SECRET_KEY=sk_test_your_secret_key_here
STRIPE_PUBLISHABLE_KEY=pk_test_your_publishable_key_here
STRIPE_PRICE_ID=price_your_price_id_here
STRIPE_WEBHOOK_SECRET=whsec_your_webhook_secret_here
```

**Production Note**: For production deployments, replace test keys (`sk_test_`, `pk_test_`) with live keys (`sk_live_`, `pk_live_`) and use the production price ID and webhook secret.

## 5. Verify Setup

To verify your Stripe integration is working correctly:

1. Start your application with the environment variables configured
2. Navigate to the subscription page (`/subscription`)
3. Click **Upgrade to Premium**
4. Use Stripe's test card number: `4242 4242 4242 4242`
   - Use any future expiration date
   - Use any 3-digit CVC
   - Use any postal code
5. Complete the checkout process
6. Verify the webhook is received:
   - Check your application logs for webhook events
   - In the Stripe Dashboard, go to **Developers > Webhooks** and check the event history

## Test Card Numbers

Stripe provides test card numbers for different scenarios:

| Card Number         | Scenario                    |
|--------------------|-----------------------------|
| 4242 4242 4242 4242 | Successful payment          |
| 4000 0000 0000 0002 | Card declined               |
| 4000 0000 0000 9995 | Insufficient funds          |
| 4000 0025 0000 3155 | Requires 3D Secure (SCA)    |

For more test cards, see: https://stripe.com/docs/testing

## Troubleshooting

### Webhook Events Not Received

- Verify your webhook endpoint URL is correct and accessible
- Check that your `STRIPE_WEBHOOK_SECRET` matches the webhook signing secret
- Review webhook event logs in the Stripe Dashboard (**Developers > Webhooks**)
- Ensure your application is listening on the correct port and path

### Checkout Session Fails

- Verify `STRIPE_PRICE_ID` matches your created price ID
- Check that your `STRIPE_SECRET_KEY` is valid and in test mode
- Review Stripe API logs in the Dashboard (**Developers > Logs**)

### Subscription Not Updating User Status

- Check webhook handler logic in `src/routes/webhooks.rs`
- Verify database updates are being executed
- Review application logs for errors during webhook processing

## Additional Resources

- [Stripe API Documentation](https://stripe.com/docs/api)
- [Stripe Subscriptions Guide](https://stripe.com/docs/billing/subscriptions/overview)
- [Stripe Webhooks Documentation](https://stripe.com/docs/webhooks)
- [Stripe Testing Guide](https://stripe.com/docs/testing)
