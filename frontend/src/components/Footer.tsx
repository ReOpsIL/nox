import React from 'react';
import './Footer.css';

/**
 * Footer component for the dashboard
 */
const Footer: React.FC = () => {
  return (
    <footer className="footer">
      <div className="footer-content">
        <div className="footer-section">
          <p className="copyright">
            &copy; {new Date().getFullYear()} Nox - Autonomous Agent Framework
          </p>
        </div>
        <div className="footer-section">
          <p className="version">Version 0.1.0</p>
        </div>
        <div className="footer-section">
          <a href="https://github.com/yourusername/nox" target="_blank" rel="noopener noreferrer" className="footer-link">
            GitHub
          </a>
          <a href="/docs" className="footer-link">
            Documentation
          </a>
          <a href="/api-docs" className="footer-link">
            API
          </a>
        </div>
      </div>
    </footer>
  );
};

export default Footer;