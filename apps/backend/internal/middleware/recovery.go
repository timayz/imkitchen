package middleware

import (
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
)

// Recovery returns a gin.HandlerFunc for recovering from panics
func Recovery() gin.HandlerFunc {
	return func(c *gin.Context) {
		defer func() {
			if err := recover(); err != nil {
				log.Printf("Panic recovered: %v", err)
				
				// Return JSON error response
				c.JSON(http.StatusInternalServerError, gin.H{
					"error": "Internal server error",
					"status": "error",
				})
				c.Abort()
			}
		}()
		
		c.Next()
	}
}