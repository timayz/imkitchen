package tests

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/imkitchen/backend/internal/services"
)

// UnitConversionServiceTestSuite test suite for unit conversion service
type UnitConversionServiceTestSuite struct {
	suite.Suite
	service *services.UnitConversionService
}

func (suite *UnitConversionServiceTestSuite) SetupTest() {
	suite.service = services.NewUnitConversionService()
}

func (suite *UnitConversionServiceTestSuite) TestConvert_SameUnit() {
	// Given
	amount := 2.0
	unit := "cup"

	// When
	result := suite.service.Convert(amount, unit, unit)

	// Then
	assert.Equal(suite.T(), amount, result)
}

func (suite *UnitConversionServiceTestSuite) TestConvert_VolumeConversions() {
	testCases := []struct {
		name       string
		amount     float64
		fromUnit   string
		toUnit     string
		expected   float64
		tolerance  float64
	}{
		{
			name:      "Cups to Tablespoons",
			amount:    1.0,
			fromUnit:  "cup",
			toUnit:    "tablespoon",
			expected:  16.0,
			tolerance: 0.01,
		},
		{
			name:      "Cups to Teaspoons",
			amount:    1.0,
			fromUnit:  "cup",
			toUnit:    "teaspoon",
			expected:  48.0,
			tolerance: 0.01,
		},
		{
			name:      "Tablespoons to Teaspoons",
			amount:    1.0,
			fromUnit:  "tablespoon",
			toUnit:    "teaspoon",
			expected:  3.0,
			tolerance: 0.01,
		},
		{
			name:      "Cups to Fluid Ounces",
			amount:    1.0,
			fromUnit:  "cup",
			toUnit:    "fluid ounce",
			expected:  8.0,
			tolerance: 0.01,
		},
		{
			name:      "Tablespoons to Milliliters",
			amount:    1.0,
			fromUnit:  "tablespoon",
			toUnit:    "milliliter",
			expected:  14.7868,
			tolerance: 0.1,
		},
	}

	for _, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			result := suite.service.Convert(tc.amount, tc.fromUnit, tc.toUnit)
			assert.InDelta(t, tc.expected, result, tc.tolerance, "Conversion from %s to %s failed", tc.fromUnit, tc.toUnit)
		})
	}
}

func (suite *UnitConversionServiceTestSuite) TestConvert_WeightConversions() {
	testCases := []struct {
		name       string
		amount     float64
		fromUnit   string
		toUnit     string
		expected   float64
		tolerance  float64
	}{
		{
			name:      "Pounds to Ounces",
			amount:    1.0,
			fromUnit:  "pound",
			toUnit:    "ounce",
			expected:  16.0,
			tolerance: 0.01,
		},
		{
			name:      "Pounds to Grams",
			amount:    1.0,
			fromUnit:  "pound",
			toUnit:    "gram",
			expected:  453.592,
			tolerance: 0.1,
		},
		{
			name:      "Ounces to Grams",
			amount:    1.0,
			fromUnit:  "ounce",
			toUnit:    "gram",
			expected:  28.3495,
			tolerance: 0.1,
		},
		{
			name:      "Kilograms to Pounds",
			amount:    1.0,
			fromUnit:  "kilogram",
			toUnit:    "pound",
			expected:  2.20462,
			tolerance: 0.01,
		},
	}

	for _, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			result := suite.service.Convert(tc.amount, tc.fromUnit, tc.toUnit)
			assert.InDelta(t, tc.expected, result, tc.tolerance, "Conversion from %s to %s failed", tc.fromUnit, tc.toUnit)
		})
	}
}

func (suite *UnitConversionServiceTestSuite) TestConvert_UnitNormalization() {
	testCases := []struct {
		name     string
		amount   float64
		fromUnit string
		toUnit   string
		expected float64
	}{
		{
			name:     "Cups to Tablespoons (plural to singular)",
			amount:   1.0,
			fromUnit: "cups",
			toUnit:   "tablespoon",
			expected: 16.0,
		},
		{
			name:     "Tbsp to Teaspoons (abbreviation)",
			amount:   1.0,
			fromUnit: "tbsp",
			toUnit:   "tsp",
			expected: 3.0,
		},
		{
			name:     "Lbs to Ounces (abbreviation)",
			amount:   1.0,
			fromUnit: "lbs",
			toUnit:   "oz",
			expected: 16.0,
		},
	}

	for _, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			result := suite.service.Convert(tc.amount, tc.fromUnit, tc.toUnit)
			assert.Equal(t, tc.expected, result, "Conversion with normalized units failed")
		})
	}
}

func (suite *UnitConversionServiceTestSuite) TestConvert_UnsupportedConversion() {
	// Given - trying to convert volume to weight
	amount := 1.0
	fromUnit := "cup"
	toUnit := "pound"

	// When
	result := suite.service.Convert(amount, fromUnit, toUnit)

	// Then - should return original amount when conversion is not possible
	assert.Equal(suite.T(), amount, result)
}

func (suite *UnitConversionServiceTestSuite) TestCanConvert_SupportedConversions() {
	testCases := []struct {
		name     string
		fromUnit string
		toUnit   string
		expected bool
	}{
		{"Cup to Tablespoon", "cup", "tablespoon", true},
		{"Pound to Ounce", "pound", "ounce", true},
		{"Tablespoon to Milliliter", "tablespoon", "milliliter", true},
		{"Same unit", "cup", "cup", true},
		{"Cup to Pound (unsupported)", "cup", "pound", false},
		{"Unknown units", "unknownunit1", "unknownunit2", false},
	}

	for _, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			result := suite.service.CanConvert(tc.fromUnit, tc.toUnit)
			assert.Equal(t, tc.expected, result)
		})
	}
}

func (suite *UnitConversionServiceTestSuite) TestGetCompatibleUnit() {
	testCases := []struct {
		name     string
		unit1    string
		unit2    string
		expected string
	}{
		{
			name:     "Same units",
			unit1:    "cup",
			unit2:    "cup",
			expected: "cup",
		},
		{
			name:     "Compatible volume units - prefer cup",
			unit1:    "tablespoon",
			unit2:    "teaspoon",
			expected: "cup",
		},
		{
			name:     "Compatible weight units - prefer pound",
			unit1:    "ounce",
			unit2:    "gram",
			expected: "pound",
		},
		{
			name:     "Incompatible units",
			unit1:    "cup",
			unit2:    "pound",
			expected: "cup", // Returns first unit when no conversion possible
		},
	}

	for _, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			result := suite.service.GetCompatibleUnit(tc.unit1, tc.unit2)
			assert.Equal(t, tc.expected, result)
		})
	}
}

func (suite *UnitConversionServiceTestSuite) TestGenerateIngredientKey() {
	testCases := []struct {
		name     string
		itemName string
		unit     string
		expected string
	}{
		{
			name:     "Basic key generation",
			itemName: "Chicken Breast",
			unit:     "pound",
			expected: "chicken breast_pound",
		},
		{
			name:     "Key with unit normalization",
			itemName: "Rice",
			unit:     "cups",
			expected: "rice_cup",
		},
		{
			name:     "Key with whitespace trimming",
			itemName: "  Olive Oil  ",
			unit:     " tbsp ",
			expected: "olive oil_tablespoon",
		},
	}

	for _, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			result := suite.service.GenerateIngredientKey(tc.itemName, tc.unit)
			assert.Equal(t, tc.expected, result)
		})
	}
}

func (suite *UnitConversionServiceTestSuite) TestComplexIngredientAggregation() {
	// Test a realistic scenario of combining multiple ingredients with different units
	ingredients := []struct {
		name   string
		amount float64
		unit   string
	}{
		{"Chicken", 1.0, "pound"},
		{"Chicken", 8.0, "ounce"},   // Should combine to 1.5 pounds
		{"Rice", 2.0, "cup"},
		{"Rice", 4.0, "tablespoon"}, // Should combine to 2.25 cups
	}

	// Test chicken aggregation (1 pound + 8 ounces = 1.5 pounds)
	chickenKey := suite.service.GenerateIngredientKey("Chicken", "pound")
	
	// Convert 8 ounces to pounds and add to 1 pound
	ouncesToPounds := suite.service.Convert(8.0, "ounce", "pound")
	totalChicken := 1.0 + ouncesToPounds

	assert.InDelta(suite.T(), 1.5, totalChicken, 0.01, "Chicken aggregation should equal 1.5 pounds")

	// Test rice aggregation (2 cups + 4 tablespoons = 2.25 cups)
	riceKey := suite.service.GenerateIngredientKey("Rice", "cup")
	
	// Convert 4 tablespoons to cups and add to 2 cups
	tablespoonsToCups := suite.service.Convert(4.0, "tablespoon", "cup")
	totalRice := 2.0 + tablespoonsToCups

	assert.InDelta(suite.T(), 2.25, totalRice, 0.01, "Rice aggregation should equal 2.25 cups")

	// Verify keys are different
	assert.NotEqual(suite.T(), chickenKey, riceKey, "Different ingredients should have different keys")
}

// TestUnitConversionServiceTestSuite runs the test suite
func TestUnitConversionServiceTestSuite(t *testing.T) {
	suite.Run(t, new(UnitConversionServiceTestSuite))
}