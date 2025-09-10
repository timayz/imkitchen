package services

import (
	"strings"
	"fmt"
)

// UnitConversionService handles unit conversions for ingredient combining
type UnitConversionService struct {
	volumeConversions map[string]map[string]float64
	weightConversions map[string]map[string]float64
	lengthConversions map[string]map[string]float64
}

// NewUnitConversionService creates a new unit conversion service with predefined conversions
func NewUnitConversionService() *UnitConversionService {
	return &UnitConversionService{
		volumeConversions: initVolumeConversions(),
		weightConversions: initWeightConversions(),
		lengthConversions: initLengthConversions(),
	}
}

// Convert attempts to convert an amount from one unit to another
func (u *UnitConversionService) Convert(amount float64, fromUnit, toUnit string) float64 {
	// Normalize units (lowercase, remove plurals)
	fromUnit = u.normalizeUnit(fromUnit)
	toUnit = u.normalizeUnit(toUnit)

	// Same unit, no conversion needed
	if fromUnit == toUnit {
		return amount
	}

	// Try volume conversion
	if conversion, exists := u.volumeConversions[fromUnit][toUnit]; exists {
		return amount * conversion
	}

	// Try weight conversion
	if conversion, exists := u.weightConversions[fromUnit][toUnit]; exists {
		return amount * conversion
	}

	// Try length conversion
	if conversion, exists := u.lengthConversions[fromUnit][toUnit]; exists {
		return amount * conversion
	}

	// No conversion found, return original amount
	return amount
}

// CanConvert checks if conversion is possible between two units
func (u *UnitConversionService) CanConvert(fromUnit, toUnit string) bool {
	fromUnit = u.normalizeUnit(fromUnit)
	toUnit = u.normalizeUnit(toUnit)

	if fromUnit == toUnit {
		return true
	}

	// Check volume conversions
	if _, exists := u.volumeConversions[fromUnit][toUnit]; exists {
		return true
	}

	// Check weight conversions
	if _, exists := u.weightConversions[fromUnit][toUnit]; exists {
		return true
	}

	// Check length conversions
	if _, exists := u.lengthConversions[fromUnit][toUnit]; exists {
		return true
	}

	return false
}

// GetCompatibleUnit returns a compatible unit for combining ingredients
func (u *UnitConversionService) GetCompatibleUnit(unit1, unit2 string) string {
	unit1 = u.normalizeUnit(unit1)
	unit2 = u.normalizeUnit(unit2)

	// If they're the same, return one of them
	if unit1 == unit2 {
		return unit1
	}

	// Check if they can be converted to each other
	if u.CanConvert(unit1, unit2) {
		// Prefer common units for combination
		preferredUnits := []string{"cup", "tablespoon", "teaspoon", "pound", "ounce", "gram", "kilogram"}
		
		for _, preferred := range preferredUnits {
			if u.CanConvert(unit1, preferred) && u.CanConvert(unit2, preferred) {
				return preferred
			}
		}
		
		// Default to unit2 if convertible
		return unit2
	}

	// Cannot convert, return unit1 (will require manual handling)
	return unit1
}

// normalizeUnit normalizes unit strings for consistent matching
func (u *UnitConversionService) normalizeUnit(unit string) string {
	unit = strings.ToLower(strings.TrimSpace(unit))
	
	// Handle common plurals and abbreviations
	normalizations := map[string]string{
		// Volume
		"cups":        "cup",
		"c":           "cup",
		"tablespoons": "tablespoon",
		"tbsp":        "tablespoon",
		"tb":          "tablespoon",
		"teaspoons":   "teaspoon",
		"tsp":         "teaspoon",
		"t":           "teaspoon",
		"fluid ounces": "fluid ounce",
		"fl oz":       "fluid ounce",
		"pints":       "pint",
		"pt":          "pint",
		"quarts":      "quart",
		"qt":          "quart",
		"gallons":     "gallon",
		"gal":         "gallon",
		"liters":      "liter",
		"l":           "liter",
		"milliliters": "milliliter",
		"ml":          "milliliter",
		
		// Weight
		"pounds":      "pound",
		"lbs":         "pound",
		"lb":          "pound",
		"ounces":      "ounce",
		"oz":          "ounce",
		"grams":       "gram",
		"g":           "gram",
		"kilograms":   "kilogram",
		"kg":          "kilogram",
		
		// Length
		"inches":      "inch",
		"in":          "inch",
		"feet":        "foot",
		"ft":          "foot",
	}

	if normalized, exists := normalizations[unit]; exists {
		return normalized
	}

	return unit
}

// initVolumeConversions initializes volume conversion mappings
func initVolumeConversions() map[string]map[string]float64 {
	return map[string]map[string]float64{
		"cup": {
			"tablespoon":  16,
			"teaspoon":    48,
			"fluid ounce": 8,
			"pint":        0.5,
			"quart":       0.25,
			"gallon":      0.0625,
			"liter":       0.236588,
			"milliliter":  236.588,
		},
		"tablespoon": {
			"cup":         0.0625,
			"teaspoon":    3,
			"fluid ounce": 0.5,
			"milliliter":  14.7868,
		},
		"teaspoon": {
			"cup":         0.0208333,
			"tablespoon":  0.333333,
			"fluid ounce": 0.166667,
			"milliliter":  4.92892,
		},
		"fluid ounce": {
			"cup":         0.125,
			"tablespoon":  2,
			"teaspoon":    6,
			"pint":        0.0625,
			"quart":       0.03125,
			"milliliter":  29.5735,
		},
		"pint": {
			"cup":         2,
			"fluid ounce": 16,
			"quart":       0.5,
			"gallon":      0.125,
			"liter":       0.473176,
		},
		"quart": {
			"cup":         4,
			"pint":        2,
			"fluid ounce": 32,
			"gallon":      0.25,
			"liter":       0.946353,
		},
		"gallon": {
			"cup":         16,
			"pint":        8,
			"quart":       4,
			"fluid ounce": 128,
			"liter":       3.78541,
		},
		"liter": {
			"cup":         4.22675,
			"milliliter":  1000,
			"gallon":      0.264172,
		},
		"milliliter": {
			"cup":         0.00422675,
			"tablespoon":  0.067628,
			"teaspoon":    0.202884,
			"fluid ounce": 0.033814,
			"liter":       0.001,
		},
	}
}

// initWeightConversions initializes weight conversion mappings
func initWeightConversions() map[string]map[string]float64 {
	return map[string]map[string]float64{
		"pound": {
			"ounce":     16,
			"gram":      453.592,
			"kilogram":  0.453592,
		},
		"ounce": {
			"pound":     0.0625,
			"gram":      28.3495,
			"kilogram":  0.0283495,
		},
		"gram": {
			"pound":     0.00220462,
			"ounce":     0.035274,
			"kilogram":  0.001,
		},
		"kilogram": {
			"pound":     2.20462,
			"ounce":     35.274,
			"gram":      1000,
		},
	}
}

// initLengthConversions initializes length conversion mappings
func initLengthConversions() map[string]map[string]float64 {
	return map[string]map[string]float64{
		"inch": {
			"foot": 0.0833333,
		},
		"foot": {
			"inch": 12,
		},
	}
}

// GenerateIngredientKey creates a unique key for ingredient aggregation
func (u *UnitConversionService) GenerateIngredientKey(name, unit string) string {
	normalizedName := strings.ToLower(strings.TrimSpace(name))
	normalizedUnit := u.normalizeUnit(unit)
	return fmt.Sprintf("%s_%s", normalizedName, normalizedUnit)
}