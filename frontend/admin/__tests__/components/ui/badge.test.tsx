import { render, screen } from '@testing-library/react'
import { Badge } from '@/components/ui/badge'

describe('Badge', () => {
  it('should render badge correctly', () => {
    render(<Badge>Test Badge</Badge>)

    expect(screen.getByText('Test Badge')).toBeInTheDocument()
  })

  it('should render different variants', () => {
    const { rerender } = render(<Badge variant="default">Default</Badge>)
    expect(screen.getByText('Default')).toBeInTheDocument()

    rerender(<Badge variant="secondary">Secondary</Badge>)
    expect(screen.getByText('Secondary')).toBeInTheDocument()

    rerender(<Badge variant="destructive">Destructive</Badge>)
    expect(screen.getByText('Destructive')).toBeInTheDocument()

    rerender(<Badge variant="outline">Outline</Badge>)
    expect(screen.getByText('Outline')).toBeInTheDocument()

    rerender(<Badge variant="success">Success</Badge>)
    expect(screen.getByText('Success')).toBeInTheDocument()

    rerender(<Badge variant="warning">Warning</Badge>)
    expect(screen.getByText('Warning')).toBeInTheDocument()

    rerender(<Badge variant="info">Info</Badge>)
    expect(screen.getByText('Info')).toBeInTheDocument()
  })

  it('should apply custom className', () => {
    render(<Badge className="custom-class">Custom</Badge>)

    expect(screen.getByText('Custom')).toHaveClass('custom-class')
  })

  it('should render children correctly', () => {
    render(
      <Badge>
        <span>Icon</span> Text
      </Badge>
    )

    expect(screen.getByText('Icon')).toBeInTheDocument()
    expect(screen.getByText(/Text/)).toBeInTheDocument()
  })
})
