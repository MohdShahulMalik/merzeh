# Merzah - Modern Islamic Community Platform

**Strengthening the Ummah through Digital Connection and Holistic Growth**

## 🌙 Overview

Merzah is a modern Islamic community platform designed to help Muslims grow in every aspect of their lives – spiritual, personal, educational, and professional. It connects users to their local mosques for real-time Iqamah times and community events, while providing a structured hub of educational resources that cover both religious and worldly knowledge.

### Vision

Our core vision is to strengthen the local Muslim community (ummah) by making the masjid both the digital and physical center of life. We aim to support users in:

- **Fulfilling their obligations** (prayer, learning their deen)
- **Building beneficial habits and skills** (career, finance, family, mental health, productivity)
- **Growing as balanced Muslims** who are strong both in deen and dunya

### Philosophy

> **Prayer grounds the day. The mosque anchors the community. Knowledge fuels long-term growth.**

## 🕌 Key Modules

### 1. Mosque & Prayer Module

**Real-time Prayer Connection**
- Administrator-updated Iqamah times for your local mosque
- Clear, mobile-friendly display of today's prayer schedule
- Next prayer countdown with notifications
- Mosque discovery with facility information
- Favorite mosque selection for personalized experience

### 2. Events & Community Life

**Centralized Community Hub**
- Upcoming events: Jumu'ah khutbah themes, weekly classes, Halaqahs
- Youth programs, charity drives, mental health workshops
- Career fairs, networking events, skill-building workshops
- Both religious (Tafsir class) and worldly (CV workshop) events
- Islamic guideline compliance for all content

### 3. Education Hub (Deen & Dunya)

**Structured Learning Tracks**

#### Faith & Worship
- Foundations of Aqeedah
- Fiqh of Salah, Zakah, Fasting, Hajj
- Quran recitation, Tajweed, Tafsir
- Prophetic character and manners
- Contemporary fiqh issues

#### Life & Wellbeing
- Islamic perspective on mental health and resilience
- Time management and habit formation
- Family relationships, marriage, parenting
- Self-discipline and personal development

#### Career & Skills
- Study skills and exam strategies
- Programming, digital skills, modern tools
- Professional communication and leadership
- Community organizing and volunteering

#### Finance & Productivity
- Halal personal finance and budgeting
- Avoiding riba and common financial pitfalls
- Entrepreneurship within Shariah limits
- Productivity tools and techniques

## 🎯 Platform Features

### User Dashboard
- Next prayer time and Iqamah schedule
- Today's community events
- Learning progress and course recommendations
- Personalized content based on interests

### Islamic Compliance
- All content reviewed for Islamic authenticity
- Scholar-approved educational materials
- Shariah-compliant financial guidance
- Moderated community discussions

### Community Engagement
- Mosque administrator tools
- Event creation and management
- Course enrollment and progress tracking
- Discussion forums with Islamic etiquette

## 🚀 Getting Started

### Prerequisites

This project uses the [Leptos](https://github.com/leptos-rs/leptos) web framework and requires:

1. **Rust nightly toolchain**
   ```bash
   rustup toolchain install nightly --allow-downgrade
   rustup target add wasm32-unknown-unknown
   ```

2. **cargo-leptos**
   ```bash
   cargo install cargo-leptos --locked
   ```

3. **Additional tools** (optional)
   ```bash
   cargo install cargo-generate
   npm install -g sass
   ```

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/merzah.git
   cd merzah
   ```

2. **Run development server**
   ```bash
   cargo leptos watch
   ```
   
   Access your local development server at: `http://localhost:3000`

3. **Build for production**
   ```bash
   cargo leptos build --release
   ```

### Project Structure

```
merzah/
├── src/
│   ├── models/          # Data structures (User, Mosque, Event, Course)
│   ├── components/      # Reusable UI components
│   ├── pages/          # Page components
│   ├── services/       # API and business logic
│   └── app.rs          # Main application entry
├── public/             # Static assets
├── style/              # CSS and styling
└── Cargo.toml          # Rust dependencies
```

## 🤝 Contributing

We welcome contributions from the Muslim community! Please ensure:

1. **Islamic Compliance**: All content must be reviewed for Islamic authenticity
2. **Code Quality**: Follow Rust best practices and Leptos conventions
3. **Community Guidelines**: Maintain respectful, Islamic etiquette in all interactions
4. **Documentation**: Update documentation for new features

### Areas of Contribution
- **Content Creation**: Educational materials, course content
- **Technical Development**: Frontend, backend, mobile features
- **Islamic Review**: Scholar review of educational content
- **Translation**: Localizing content for different languages
- **Community Moderation**: Forum and event moderation

## 📋 Community Guidelines

### Islamic Etiquette
- Maintain respect and good manners (adab) in all interactions
- Avoid controversial topics and sectarian discussions
- Focus on beneficial knowledge and constructive dialogue
- Follow Quranic principles: "Speak good or remain silent"

### Content Standards
- All educational content must be reviewed by qualified scholars
- Financial advice must be Shariah-compliant
- Events must align with Islamic values
- No inappropriate content or discussions

## 📞 Contact & Support

**Community Support**: support@merzah.org  
**Islamic Queries**: scholars@merzah.org  
**Technical Issues**: dev@merzah.org  

**Social Media**:
- Twitter: [@merzah_app](https://twitter.com/merzah_app)
- Instagram: [@merzah_app](https://instagram.com/merzah_app)
- LinkedIn: [Merzah](https://linkedin.com/company/merzah)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Leptos Framework**: For providing the robust web framework
- **Islamic Scholars**: For reviewing and authenticating educational content
- **Community Contributors**: For their time, knowledge, and dedication
- **Open Source Community**: For the tools and libraries that make this possible

---

**May this platform be a means of bringing the ummah closer to Allah and to each other.**

*"And cooperate in righteousness and piety, but do not cooperate in sin and aggression."* - Quran 5:2
