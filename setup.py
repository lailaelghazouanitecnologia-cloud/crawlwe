"""
Alternative setup.py for Python-only installation (without Rust).

Use this if you have trouble building the Rust extension:
    pip install -e . --no-build-isolation

Or install dependencies manually:
    pip install click rich beautifulsoup4 lxml cssbeautifier requests
    pip install -e . --no-build-isolation
"""

from setuptools import setup, find_packages

setup(
    name="crawlwe",
    version="0.1.0",
    description="Web page extractor for ML training - captures HTML + CSS",
    author="CrawlWe Team",
    python_requires=">=3.10",
    packages=find_packages(exclude=["rust", "scripts", "data"]),
    install_requires=[
        "click>=8.1.0",
        "rich>=13.0.0",
        "beautifulsoup4>=4.12.0",
        "lxml>=5.0.0",
        "cssbeautifier>=1.14.0",
        "requests>=2.31.0",
    ],
    extras_require={
        "full": [
            "aiohttp>=3.9.0",
            "aiofiles>=23.0.0",
            "pydantic>=2.5.0",
        ],
        "dev": [
            "pytest>=7.0.0",
            "black>=24.0.0",
            "ruff>=0.1.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "crawlwe=crawlwe.cli:main",
        ],
    },
)
