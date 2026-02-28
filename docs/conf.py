# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'KoproGo'
copyright = '2025, KoproGo Team'
author = 'KoproGo Team'
release = '0.1.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.napoleon',
    'sphinx.ext.viewcode',
    'sphinx.ext.intersphinx',
    'sphinx.ext.todo',
    'myst_parser',
    'ablog',
    'sphinx_design',
]

# Support for Markdown files
source_suffix = {
    '.rst': 'restructuredtext',
    '.md': 'markdown',
}

templates_path = ['_templates']
exclude_patterns = [
    '_build',
    'Thumbs.db',
    '.DS_Store',
    '.venv',
    '.venv/**',
    '**/.venv',
    '**/.venv/**',
]

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']

html_theme_options = {
    'navigation_depth': 3,
    'collapse_navigation': False,
    'sticky_navigation': True,
    'includehidden': True,
    'titles_only': False,
    'prev_next_buttons_location': 'bottom',
    'style_external_links': False,
    'display_version': False,
}

# Disable local TOC in sidebar
html_sidebars = {
    '**': ['globaltoc.html', 'searchbox.html']
}

# -- Extension configuration -------------------------------------------------

# Napoleon settings for Google/NumPy style docstrings
napoleon_google_docstring = True
napoleon_numpy_docstring = True
napoleon_include_init_with_doc = False
napoleon_include_private_with_doc = False
napoleon_include_special_with_doc = True
napoleon_use_admonition_for_examples = False
napoleon_use_admonition_for_notes = False
napoleon_use_admonition_for_references = False
napoleon_use_ivar = False
napoleon_use_param = True
napoleon_use_rtype = True
napoleon_preprocess_types = False
napoleon_type_aliases = None
napoleon_attr_annotations = True

# Intersphinx mapping
intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
}

# MyST parser options
myst_enable_extensions = [
    "colon_fence",
    "deflist",
    "tasklist",
]

# Todo extension
todo_include_todos = True

# -- ABlog configuration -----------------------------------------------------
blog_path = 'blog'
blog_title = 'Blog KoproGo'
blog_baseurl = 'https://doc.koprogo.com'
blog_post_pattern = 'blog/*.rst'
post_auto_excerpt = 1
post_auto_image = 0
blog_feed_fulltext = False
blog_feed_length = 0
fontawesome_included = True
