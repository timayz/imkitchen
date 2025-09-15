-- CreateEnum
CREATE TYPE "storage_location" AS ENUM ('pantry', 'refrigerator', 'freezer');

-- CreateEnum
CREATE TYPE "inventory_category" AS ENUM ('proteins', 'vegetables', 'fruits', 'grains', 'dairy', 'spices', 'condiments', 'beverages', 'baking', 'frozen');

-- CreateTable
CREATE TABLE "custom_categories" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "name" VARCHAR(50) NOT NULL,
    "color" VARCHAR(7) NOT NULL,
    "icon" VARCHAR(50) NOT NULL,
    "household_id" UUID NOT NULL,
    "created_by" UUID NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL,

    CONSTRAINT "custom_categories_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "inventory_items" (
    "id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "name" VARCHAR(255) NOT NULL,
    "quantity" DECIMAL(10,2) NOT NULL,
    "unit" VARCHAR(50) NOT NULL,
    "category" VARCHAR(50) NOT NULL,
    "location" "storage_location" NOT NULL,
    "expiration_date" DATE,
    "purchase_date" DATE DEFAULT CURRENT_TIMESTAMP,
    "estimated_cost" DECIMAL(10,2),
    "household_id" UUID NOT NULL,
    "added_by" UUID NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL,

    CONSTRAINT "inventory_items_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE INDEX "idx_custom_categories_household" ON "custom_categories"("household_id");

-- CreateIndex
CREATE UNIQUE INDEX "custom_categories_name_household_id_key" ON "custom_categories"("name", "household_id");

-- CreateIndex
CREATE INDEX "idx_inventory_household_location" ON "inventory_items"("household_id", "location");

-- CreateIndex
CREATE INDEX "idx_inventory_household_category" ON "inventory_items"("household_id", "category");

-- CreateIndex
CREATE INDEX "idx_inventory_expiration" ON "inventory_items"("expiration_date");

-- AddForeignKey
ALTER TABLE "custom_categories" ADD CONSTRAINT "custom_categories_household_id_fkey" FOREIGN KEY ("household_id") REFERENCES "households"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "custom_categories" ADD CONSTRAINT "custom_categories_created_by_fkey" FOREIGN KEY ("created_by") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "inventory_items" ADD CONSTRAINT "inventory_items_household_id_fkey" FOREIGN KEY ("household_id") REFERENCES "households"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "inventory_items" ADD CONSTRAINT "inventory_items_added_by_fkey" FOREIGN KEY ("added_by") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
