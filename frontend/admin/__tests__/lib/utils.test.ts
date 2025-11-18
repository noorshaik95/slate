import {
  cn,
  formatBytes,
  formatNumber,
  formatCurrency,
  formatPercentage,
  truncate,
  debounce,
  getInitials,
  getStatusColor,
} from '@/lib/utils'

describe('Utils', () => {
  describe('cn', () => {
    it('should merge class names correctly', () => {
      expect(cn('class1', 'class2')).toBe('class1 class2')
    })

    it('should handle conditional classes', () => {
      expect(cn('class1', false && 'class2', 'class3')).toBe('class1 class3')
    })

    it('should merge tailwind classes correctly', () => {
      expect(cn('px-2', 'px-4')).toBe('px-4')
    })
  })

  describe('formatBytes', () => {
    it('should format 0 bytes', () => {
      expect(formatBytes(0)).toBe('0 Bytes')
    })

    it('should format bytes', () => {
      expect(formatBytes(100)).toBe('100 Bytes')
    })

    it('should format KB', () => {
      expect(formatBytes(1024)).toBe('1 KB')
    })

    it('should format MB', () => {
      expect(formatBytes(1048576)).toBe('1 MB')
    })

    it('should format GB', () => {
      expect(formatBytes(1073741824)).toBe('1 GB')
    })

    it('should format with custom decimals', () => {
      expect(formatBytes(1536, 0)).toBe('2 KB')
    })
  })

  describe('formatNumber', () => {
    it('should format numbers with commas', () => {
      expect(formatNumber(1000)).toBe('1,000')
      expect(formatNumber(1000000)).toBe('1,000,000')
    })

    it('should format small numbers', () => {
      expect(formatNumber(100)).toBe('100')
    })
  })

  describe('formatCurrency', () => {
    it('should format USD by default', () => {
      expect(formatCurrency(100)).toBe('$100.00')
      expect(formatCurrency(1234.56)).toBe('$1,234.56')
    })

    it('should format other currencies', () => {
      expect(formatCurrency(100, 'EUR')).toContain('100')
    })
  })

  describe('formatPercentage', () => {
    it('should format percentages', () => {
      expect(formatPercentage(50)).toBe('50.0%')
      expect(formatPercentage(33.333)).toBe('33.3%')
    })

    it('should format with custom decimals', () => {
      expect(formatPercentage(33.333, 2)).toBe('33.33%')
    })
  })

  describe('truncate', () => {
    it('should truncate long strings', () => {
      expect(truncate('This is a long string', 10)).toBe('This is a ...')
    })

    it('should not truncate short strings', () => {
      expect(truncate('Short', 10)).toBe('Short')
    })
  })

  describe('debounce', () => {
    jest.useFakeTimers()

    it('should debounce function calls', () => {
      const mockFn = jest.fn()
      const debouncedFn = debounce(mockFn, 1000)

      debouncedFn()
      debouncedFn()
      debouncedFn()

      expect(mockFn).not.toHaveBeenCalled()

      jest.runAllTimers()

      expect(mockFn).toHaveBeenCalledTimes(1)
    })

    jest.useRealTimers()
  })

  describe('getInitials', () => {
    it('should get initials from name', () => {
      expect(getInitials('John Doe')).toBe('JD')
      expect(getInitials('Jane Smith')).toBe('JS')
    })

    it('should limit to 2 characters', () => {
      expect(getInitials('John Middle Doe')).toBe('JM')
    })

    it('should handle single names', () => {
      expect(getInitials('John')).toBe('J')
    })
  })

  describe('getStatusColor', () => {
    it('should return correct color for success', () => {
      expect(getStatusColor('success')).toContain('green')
    })

    it('should return correct color for error', () => {
      expect(getStatusColor('error')).toContain('red')
    })

    it('should return correct color for warning', () => {
      expect(getStatusColor('warning')).toContain('yellow')
    })

    it('should return default color for unknown status', () => {
      expect(getStatusColor('unknown')).toContain('blue')
    })

    it('should be case insensitive', () => {
      expect(getStatusColor('SUCCESS')).toContain('green')
    })
  })
})
