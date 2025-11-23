import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Input } from '@/components/ui/input'

describe('Input', () => {
  it('should render input correctly', () => {
    render(<Input placeholder="Enter text" />)

    expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument()
  })

  it('should handle text input', async () => {
    const user = userEvent.setup()
    render(<Input placeholder="Enter text" />)

    const input = screen.getByPlaceholderText('Enter text')

    await user.type(input, 'Hello World')

    expect(input).toHaveValue('Hello World')
  })

  it('should handle onChange events', async () => {
    const user = userEvent.setup()
    const handleChange = jest.fn()

    render(<Input placeholder="Enter text" onChange={handleChange} />)

    const input = screen.getByPlaceholderText('Enter text')

    await user.type(input, 'Test')

    expect(handleChange).toHaveBeenCalled()
  })

  it('should be disabled when disabled prop is true', () => {
    render(<Input placeholder="Enter text" disabled />)

    expect(screen.getByPlaceholderText('Enter text')).toBeDisabled()
  })

  it('should render different input types', () => {
    const { rerender } = render(<Input type="text" placeholder="Text" />)
    expect(screen.getByPlaceholderText('Text')).toHaveAttribute('type', 'text')

    rerender(<Input type="email" placeholder="Email" />)
    expect(screen.getByPlaceholderText('Email')).toHaveAttribute('type', 'email')

    rerender(<Input type="password" placeholder="Password" />)
    expect(screen.getByPlaceholderText('Password')).toHaveAttribute('type', 'password')

    rerender(<Input type="number" placeholder="Number" />)
    expect(screen.getByPlaceholderText('Number')).toHaveAttribute('type', 'number')
  })

  it('should apply custom className', () => {
    render(<Input placeholder="Test" className="custom-class" />)

    expect(screen.getByPlaceholderText('Test')).toHaveClass('custom-class')
  })

  it('should handle value prop', () => {
    render(<Input value="Initial Value" readOnly />)

    expect(screen.getByDisplayValue('Initial Value')).toBeInTheDocument()
  })
})
