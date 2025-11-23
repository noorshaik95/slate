import { renderHook, act } from '@testing-library/react'
import { useToast, toast } from '@/hooks/use-toast'

describe('useToast', () => {
  beforeEach(() => {
    jest.clearAllTimers()
  })

  it('should initialize with empty toasts', () => {
    const { result } = renderHook(() => useToast())

    expect(result.current.toasts).toEqual([])
  })

  it('should add a toast', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({
        title: 'Test Toast',
        description: 'Test Description',
      })
    })

    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0]).toMatchObject({
      title: 'Test Toast',
      description: 'Test Description',
    })
  })

  it('should dismiss a toast', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string

    act(() => {
      const { id } = result.current.toast({
        title: 'Test Toast',
      })
      toastId = id
    })

    expect(result.current.toasts).toHaveLength(1)

    act(() => {
      result.current.dismiss(toastId)
    })

    // Toast should be marked as closed
    expect(result.current.toasts[0].open).toBe(false)
  })

  it('should limit number of toasts', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({ title: 'Toast 1' })
      result.current.toast({ title: 'Toast 2' })
      result.current.toast({ title: 'Toast 3' })
    })

    // Should only keep the most recent toast (TOAST_LIMIT = 1)
    expect(result.current.toasts).toHaveLength(1)
  })

  it('should update a toast', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string
    let updateFn: (props: any) => void

    act(() => {
      const { id, update } = result.current.toast({
        title: 'Original Title',
      })
      toastId = id
      updateFn = update
    })

    act(() => {
      updateFn({
        title: 'Updated Title',
      })
    })

    expect(result.current.toasts[0].title).toBe('Updated Title')
  })

  it('should handle toast variants', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({
        title: 'Error',
        variant: 'destructive',
      })
    })

    expect(result.current.toasts[0].variant).toBe('destructive')
  })
})

describe('toast function', () => {
  it('should create a toast directly', () => {
    const result = toast({
      title: 'Direct Toast',
      description: 'Created directly',
    })

    expect(result).toHaveProperty('id')
    expect(result).toHaveProperty('dismiss')
    expect(result).toHaveProperty('update')
  })

  it('should allow updating toast', () => {
    const { update } = toast({
      title: 'Original',
    })

    expect(() => {
      update({
        title: 'Updated',
      })
    }).not.toThrow()
  })

  it('should allow dismissing toast', () => {
    const { dismiss } = toast({
      title: 'Test',
    })

    expect(() => {
      dismiss()
    }).not.toThrow()
  })
})
