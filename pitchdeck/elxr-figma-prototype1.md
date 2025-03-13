# ELXR (Elixir Chain) Figma Prototype Documentation

## Overview

This document provides detailed specifications for the ELXR user interface prototype. The design utilizes a warm gold-amber color palette to reflect the nature of Kombucha production and implements the "vessel as block" visual metaphor throughout the interface. The prototype demonstrates the complete user journey from identity verification to fermentation monitoring and reward management.

## Design System

### Color Palette

- **Primary Amber**: #FF9800 (Deep amber for headers and primary actions)
- **Secondary Gold**: #FFC107 (Medium gold for secondary elements and accents)
- **Tertiary Amber**: #FFD54F (Light amber for backgrounds and highlights)
- **Background Cream**: #FFF8E1 (Very light cream for page backgrounds)
- **Accent Burgundy**: #C2185B (Used sparingly for alerts and special callouts)
- **Neutral Dark**: #3E2723 (For text and icons)
- **Neutral Light**: #EFEBE9 (For dividers and secondary backgrounds)

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
  - Kombucha vessels
  - Fermentation metrics (temperature, pH, alcohol content, etc.)
  - SCOBY (Symbiotic Culture of Bacteria and Yeast)
  - Bottling processes
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
- Animated Kombucha fermentation visualization

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
- Quick action buttons (add production node, bottle, stake)
- Recent activity feed
- Network status indicators
- Price oracle feed showing current Kombucha valuation

#### 2.2 Vessel Visualization
- Interactive "vessel as block" visualization showing:
  - Fill level corresponding to "block height"
  - Color gradient indicating fermentation stage
  - Animated bubbles showing active fermentation
  - Real-time sensor data overlay
  - Historical fermentation chart

#### 2.3 Notification Center
- Alerts for optimal bottling times
- Fermentation parameter warnings
- Reward distribution notifications
- Governance proposal alerts
- System updates

### 3. Production Management

#### 3.1 Production Node Registration
- Add new production vessel interface
- NFT selection for SCOBY certification
- Sensor configuration wizard
- Location and environmental data input
- Estimated reward projections

#### 3.2 Fermentation Monitoring
- Real-time telemetry dashboard showing:
  - Temperature graph with optimal range
  - pH level indicator
  - Alcohol content meter
  - Sugar content visualization
  - Microbial activity calculation
  - Bottling countdown timer

#### 3.3 Bottling Interface
- Bottling planning calculator
- Step-by-step bottling guide
- Post-bottling maintenance checklist
- Reward estimation based on volume and quality
- Bottling history log

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
- Options for direct Kombucha delivery
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

The main dashboard provides at-a-glance information on fermentation status and value:

```
┌─────────────────────────────────────────────────┐
│ ELXR                                   🔔 👤 ⋮  │
├─────────────────────────────────────────────────┤
│                                                 │
│  Production Overview              Add Vessel +   │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐   │
│  │           │  │           │  │           │   │
│  │  Active   │  │  Total    │  │  Quality  │   │
│  │  Vessels  │  │  Volume   │  │  Score    │   │
│  │