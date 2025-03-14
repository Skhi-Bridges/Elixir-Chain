# NRSH (Nourish Chain) Figma Prototype Documentation

## Overview

This document provides detailed specifications for the NRSH user interface prototype. The design utilizes a green color palette to reflect the nature of Spirulina production and implements the "tote as block" visual metaphor throughout the interface. The prototype demonstrates the complete user journey from identity verification to production monitoring and reward management.

## Design System

### Color Palette

- **Primary Green**: #106B03 (Dark emerald green for headers and primary actions)
- **Secondary Green**: #4CAF50 (Medium green for secondary elements and accents)
- **Tertiary Green**: #8BC34A (Light green for backgrounds and highlights)
- **Background Green**: #F1F8E9 (Very light green for page backgrounds)
- **Accent Blue**: #03A9F4 (Used sparingly for alerts and special callouts)
- **Neutral Dark**: #263238 (For text and icons)
- **Neutral Light**: #ECEFF1 (For dividers and secondary backgrounds)

### Typography

- **Headings**: Montserrat Bold
  - H1: 32px
  - H2: 24px
  - H3: 20px
  - H4: 18px
- **Body**: Roboto Regular
  - Body: 16px
  - Small: 14px
  - Caption: 12px
- **Monospace**: Roboto Mono (for data values and metrics)

### Iconography

- Custom icons representing:
  - Spirulina totes
  - Cultivation metrics (temperature, pH, light, etc.)
  - Harvesting processes
  - Quality certification
  - Token staking and rewards

### Component Library

- Buttons (Primary, Secondary, Tertiary, Ghost)
- Input fields
- Dropdown selectors
- Toggle switches
- Cards and panels
- Data visualization components
- Progress indicators
- Modal dialogs
- Navigation elements

## User Flows

### 1. Onboarding and Identity Verification

#### 1.1 Welcome Screen
- App introduction with mission statement
- Options to sign in or create account
- Animated Spirulina visualization

#### 1.2 Account Creation
- Email and password fields
- Terms of service and privacy policy acceptance

#### 1.3 Timesafe KYC Process
- Introduction to quantum-resistant identity verification
- Connection to Remote Online Notary service
- Document upload interface
- Biometric capture (facial recognition, optional palm scan)
- Document verification status tracking

#### 1.4 Wallet Integration
- Options to create new wallet or connect existing
- Display of public key with copy function
- Backup seed phrase process
- Multi-factor authentication setup

### 2. Dashboard

#### 2.1 Main Dashboard
- Overview statistics (total production, staked value, rewards)
- Quick action buttons (add production node, harvest, stake)
- Recent activity feed
- Network status indicators
- Price oracle feed showing current Spirulina valuation

#### 2.2 Tote Visualization
- Interactive "tote as block" visualization showing:
  - Fill level corresponding to "block height"
  - Color gradient indicating quality level
  - Animated ripples showing active growth
  - Real-time sensor data overlay
  - Historical growth chart

#### 2.3 Notification Center
- Alerts for optimal harvesting times
- Quality metric warnings
- Reward distribution notifications
- Governance proposal alerts
- System updates

### 3. Production Management

#### 3.1 Production Node Registration
- Add new production node interface
- NFT selection for culture certification
- Sensor configuration wizard
- Location and environmental data input
- Estimated reward projections

#### 3.2 Production Monitoring
- Real-time telemetry dashboard showing:
  - Temperature graph with optimal range
  - pH level indicator
  - Light intensity meter
  - Nutrient composition visualization
  - Growth rate calculation
  - Harvesting countdown timer

#### 3.3 Harvesting Interface
- Harvest planning calculator
- Step-by-step harvesting guide
- Post-harvest maintenance checklist
- Reward estimation based on volume and quality
- Harvest history log

### 4. Staking and Rewards

#### 4.1 Staking Dashboard
- Current staked assets overview
- Staking options and lock periods
- Expected return calculator
- Royalty allocation visualization
- Unstaking countdown timer

#### 4.2 Reward Management
- Reward history with filtering options
- Reward distribution schedule
- Compound vs. withdraw options
- Tax reporting assistance
- Performance analytics

#### 4.3 Market Access
- Options for direct Spirulina delivery
- Token exchange interface
- Precious metal redemption process
- Local warehouse finder
- Delivery tracking

### 5. Governance and Community

#### 5.1 Governance Portal
- Active proposals list
- Voting interface with token weight display
- Proposal creation wizard
- Governance statistics
- Execution timelock visualization

#### 5.2 Community Hub
- Knowledge base and best practices
- Forum integration
- Events calendar
- Support chat
- Producer directory

## Key Screens (Detailed)

### Dashboard Screen

The main dashboard provides at-a-glance information on production status and value:

```
┌─────────────────────────────────────────────────┐
│ NRSH                                   🔔 👤 ⋮  │
├─────────────────────────────────────────────────┤
│                                                 │
│  Production Overview               Add Node +   │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐   │
│  │           │  │           │  │           │   │
│  │  Active   │  │  Total    │  │  Quality  │   │
│  │  Nodes    │  │  Volume   │  │  Score    │   │
│  │           │  │           │  │           │   │
│  │     2     │  │  425 gal  │  │   92/100  │   │
│  └───────────┘  └───────────┘  └───────────┘   │
│                                                 │
│  Tote Visualization                        ⟳   │
│  ┌─────────────────────────────────────────┐   │
│  │                                         │   │
│  │  ┌─────────┐        ┌─────────┐        │   │
│  │  │         │        │         │        │   │
│  │  │  Tote 1 │        │  Tote 2 │        │   │
│  │  │  250gal │        │  175gal │        │   │
│  │  │   93%   │        │   87%   │        │   │
│  │  │ ████████│        │ ███████ │        │   │
│  │  │ ████████│        │ ███████ │        │   │
│  │  │ ████████│        │ ███████ │        │   │
│  │  │ ████████│        │ ███▒▒▒▒ │        │   │
│  │  │         │        │         │        │   │
│  │  └─────────┘        └─────────┘        │   │
│  │                                         │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Telemetry                                      │
│  ┌─────────────────────────────────────────┐   │
│  │ Temperature ●●●●●●●○○○  25.2°C          │   │
│  │ pH          ●●●●●●●●●○  9.8             │   │
│  │ Light       ●●●●●●○○○○  12500 lux       │   │
│  │ Density     ●●●●●●●●○○  1.03 g/cm³      │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Next Harvest                 Staking           │
│  ┌───────────────┐           ┌───────────────┐ │
│  │ Tote 1: 3d 2h │           │ Total: $83K   │ │
│  │ Tote 2: 5d 8h │           │ APY: 12.3%    │ │
│  └───────────────┘           └───────────────┘ │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Production Node Detail Screen

Detailed view of a specific production node (tote) with comprehensive metrics:

```
┌─────────────────────────────────────────────────┐
│ < Back to Dashboard               🔔 👤 ⋮       │
├─────────────────────────────────────────────────┤
│                                                 │
│  Tote 1 - Premium Spirulina            Edit ✏  │
│  NFT: Gold #3782                     Harvest 🔄 │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │                                         │   │
│  │  ╔═════════════════════════════════╗    │   │
│  │  ║                                 ║    │   │
│  │  ║                                 ║    │   │
│  │  ║                                 ║    │   │
│  │  ║        [Tote Visualization]     ║    │   │
│  │  ║         with animated           ║    │   │
│  │  ║         green liquid            ║    │   │
│  │  ║         showing current         ║    │   │
│  │  ║         fill level (93%)        ║    │   │
│  │  ║                                 ║    │   │
│  │  ╚═════════════════════════════════╝    │   │
│  │                                         │   │
│  │  Volume: 250/275 gallons                │   │
│  │  Value: $83,250                         │   │
│  │                                         │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Real-time Metrics                              │
│  ┌─────────────────────────────────────────┐   │
│  │                                         │   │
│  │  Temperature                            │   │
│  │  ┌───────────────────────────────────┐ │   │
│  │  │                                   │ │   │
│  │  │         [Line chart showing       │ │   │
│  │  │          temperature over time]    │ │   │
│  │  │                                   │ │   │
│  │  └───────────────────────────────────┘ │   │
│  │  Current: 25.2°C    Optimal: 25-30°C   │   │
│  │                                         │   │
│  │  pH Level                               │   │
│  │  ┌───────────────────────────────────┐ │   │
│  │  │                                   │ │   │
│  │  │          [pH gauge with          │ │   │
│  │  │           current level]          │ │   │
│  │  │                                   │ │   │
│  │  └───────────────────────────────────┘ │   │
│  │  Current: 9.8       Optimal: 9.5-10.5  │   │
│  │                                         │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Growth Statistics                              │
│  ┌─────────────────────────────────────────┐   │
│  │ Daily Growth: 12.8g/L       ↑8% w/w     │   │
│  │ Quality Score: 93/100       ↑2 pts      │   │
│  │ Days Since Last Harvest: 24             │   │
│  │ Estimated Harvest Value: $41,625        │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Sensor Status: All Systems Normal ✓            │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Harvesting Interface

Step-by-step interface for the harvesting process:

```
┌─────────────────────────────────────────────────┐
│ < Back to Tote 1                    🔔 👤 ⋮     │
├─────────────────────────────────────────────────┤
│                                                 │
│  Harvest Process                                │
│  ┌─────────────────────────────────────────┐   │
│  │                                         │   │
│  │  Step 2 of 5: Harvest Volume           │   │
│  │  ───○───●───○───○───○                  │   │
│  │                                         │   │
│  │  Select Volume to Harvest:              │   │
│  │                                         │   │
│  │  ┌──────────────────────────────────┐  │   │
│  │  │        125        gallons        │  │   │
│  │  └──────────────────────────────────┘  │   │
│  │        ▲            ▼                  │   │
│  │                                         │   │
│  │  Current Volume: 250 gallons           │   │
│  │  Remaining After Harvest: 125 gallons  │   │
│  │  Harvest Value: $41,625                │   │
│  │                                         │   │
│  │  [Tote visualization showing           │   │
│  │   current and post-harvest levels]     │   │
│  │                                         │   │
│  │  Makeup Mix Required:                   │   │
│  │  • Potassium Nitrate: 62.5g            │   │
│  │  • Iron Supplement: 12.5g              │   │
│  │  • Purified Water: 125 gallons         │   │
│  │                                         │   │
│  │                                         │   │
│  │         Back           Continue         │   │
│  │                                         │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Staking Interface

Interface for staking harvested Spirulina:

```
┌─────────────────────────────────────────────────┐
│ < Back to Dashboard               🔔 👤 ⋮       │
├─────────────────────────────────────────────────┤
│                                                 │
│  Stake Spirulina                                │
│  ┌─────────────────────────────────────────┐   │
│  │                                         │   │
│  │  Available to Stake: 125 gallons        │   │
│  │  Estimated Value: $41,625               │   │
│  │                                         │   │
│  │  Select Amount to Stake:                │   │
│  │  ┌──────────────────────────────────┐  │   │
│  │  │        125        gallons        │  │   │
│  │  └──────────────────────────────────┘  │   │
│  │        ▲            ▼                  │   │
│  │                                         │   │
│  │  Select Lock Period:                    │   │
│  │  ○ 30 days  (8% APY)                   │   │
│  │  ○ 90 days  (12% APY)                  │   │
│  │  ● 180 days (15% APY)                  │   │
│  │  ○ 365 days (18% APY)                  │   │
│  │                                         │   │
│  │  Estimated Rewards: $3,121.88          │   │
│  │                                         │   │
│  │  Royalty (0.999%): $415.83             │   │
│  │                                         │   │
│  │  Net Stake Value: $41,209.17           │   │
│  │                                         │   │
│  │  Unlock Date: September 30, 2025        │   │
│  │                                         │   │
│  │                                         │   │
│  │            Confirm Stake                │   │
│  │                                         │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Timesafe KYC Screen

Interface for the quantum-resistant identity verification process:

```
┌─────────────────────────────────────────────────┐
│ < Back                                          │
├─────────────────────────────────────────────────┤
│                                                 │
│  Timesafe KYC Verification                      │
│  ┌─────────────────────────────────────────┐   │
│  │                                         │   │
│  │  Step 2 of 4: Remote Notarization       │   │
│  │  ───○───●───○───○                      │   │
│  │                                         │   │
│  │  Connect with Licensed Notary            │   │
│  │                                         │   │
│  │  Your session will be encrypted using   │   │
│  │  quantum-resistant CRYSTALS-Dilithium   │   │
│  │  cryptography.                          │   │
│  │                                         │   │
│  │  Have ready:                            │   │
│  │  • Government-issued photo ID           │   │
│  │  • Proof of address                      │   │
│  │  • Social security/tax ID number        │   │
│  │                                         │   │
│  │  The notary will verify your identity   │   │
│  │  via video and create a tamper-proof    │   │
│  │  digital certificate secured with       │   │
│  │  post-quantum cryptography.             │   │
│  │                                         │   │
│  │                                         │   │
│  │        Connect to Notary Now            │   │
│  │                                         │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  Your data remains private and secured against  │
│  both classical and quantum attacks.            │
│                                                 │
└─────────────────────────────────────────────────┘
```

## Responsive Design

The interface is designed to be responsive across multiple device types:

1. **Desktop**: Full-featured dashboard with comprehensive visualizations
2. **Tablet**: Optimized layout with focus on production monitoring and key actions
3. **Mobile**: Streamlined interface prioritizing real-time alerts and essential metrics

## Accessibility Considerations

1. **Color Contrast**: All text meets WCAG AA standards for readability
2. **Screen Reader Support**: All interactive elements include appropriate ARIA labels
3. **Keyboard Navigation**: Complete functionality available through keyboard controls
4. **Text Scaling**: Interface accommodates increased text sizes without breaking layouts
5. **Alternative Text**: All images and visualizations include descriptive alternative text

## Design Principles

The UI design follows these core principles:

1. **Clarity**: Clear representation of complex data through intuitive visualizations
2. **Consistency**: Uniform patterns and interactions throughout the interface
3. **Efficiency**: Optimized workflows for common tasks with minimal steps
4. **Education**: Contextual guidance and tooltips to help users understand the system
5. **Beauty**: Aesthetically pleasing design that reflects the organic nature of Spirulina

## Next Steps

1. **User Testing**: Validate interface with target user groups
2. **Interaction Refinement**: Optimize animations and transitions
3. **Performance Optimization**: Ensure smooth operation with real-time data updates
4. **Localization**: Prepare for multilingual support
5. **Integration Testing**: Validate connections with backend systems and smart contracts

## Conclusion

The NRSH Figma prototype demonstrates a comprehensive, intuitive interface for managing Spirulina production on the blockchain. The design incorporates the "tote as block" visual metaphor throughout, creating a clear connection between physical production and blockchain concepts. The green color palette and organic visual elements reflect the nature of Spirulina cultivation while maintaining professional clarity and usability.
