#!/usr/bin/env python3
"""
Apply OpenAPI Annotations to Handler Files

This script automatically inserts utoipa::path annotations into handler files.
It backs up the original files before modification.

Usage:
    python3 scripts/apply_openapi_annotations.py [--dry-run] [--file=FILENAME]

Options:
    --dry-run       Show what would be changed without modifying files
    --file=FILE     Only process specific handler file (e.g., auth_handlers.rs)
    --backup        Create .bak backup files (default: true)
"""

import re
import sys
import shutil
from pathlib import Path
from typing import List, Optional
from generate_openapi_annotations import OpenAPIGenerator, HandlerInfo


class AnnotationApplier:
    """Applies OpenAPI annotations to handler files"""

    def __init__(self, dry_run: bool = False, create_backup: bool = True):
        self.dry_run = dry_run
        self.create_backup = create_backup
        self.generator = OpenAPIGenerator(None)
        self.modified_files = []
        self.failed_files = []

    def apply_to_file(self, file_path: Path, handlers: List[HandlerInfo]) -> bool:
        """Apply annotations to a single file"""
        try:
            content = file_path.read_text()
            original_content = content

            # Process each handler
            for handler in handlers:
                content = self._insert_annotation(content, handler)

            # Check if content changed
            if content == original_content:
                print(f"  ⏭️  No changes needed for {file_path.name}")
                return True

            # Backup original file
            if self.create_backup and not self.dry_run:
                backup_path = file_path.with_suffix(file_path.suffix + '.bak')
                shutil.copy2(file_path, backup_path)
                print(f"  💾 Backup created: {backup_path.name}")

            # Write modified content
            if not self.dry_run:
                file_path.write_text(content)
                print(f"  ✅ Modified {file_path.name} ({len(handlers)} handlers annotated)")
            else:
                print(f"  🔍 [DRY RUN] Would modify {file_path.name} ({len(handlers)} handlers)")

            self.modified_files.append(file_path)
            return True

        except Exception as e:
            print(f"  ❌ Error processing {file_path.name}: {e}")
            self.failed_files.append((file_path, str(e)))
            return False

    def _insert_annotation(self, content: str, handler: HandlerInfo) -> str:
        """Insert annotation before handler function"""
        # Generate annotation
        annotation = self.generator.generate_annotation(handler)

        # Pattern to find the handler function
        # Look for: #[method("path")] followed by pub async fn function_name
        method_lower = handler.method.lower()
        escaped_path = re.escape(handler.path)

        # Pattern: #[get("/path")] or similar actix macro
        pattern = rf'(#\[{method_lower}\("{escaped_path}"\)\]\s*\n)(pub async fn {handler.function_name}\()'

        # Check if annotation already exists
        if f'#[utoipa::path(' in content:
            # Check if this specific handler already has annotation
            check_pattern = rf'#\[utoipa::path\([^)]*\)\]\s*#\[{method_lower}\("{escaped_path}"\)\]'
            if re.search(check_pattern, content, re.MULTILINE | re.DOTALL):
                return content  # Already annotated

        # Insert annotation before the actix macro
        def replace_func(match):
            actix_macro = match.group(1)
            function_def = match.group(2)
            return f"{annotation}\n{actix_macro}{function_def}"

        new_content = re.sub(pattern, replace_func, content, count=1)

        # If pattern didn't match, try without actix macro (some handlers might not have it)
        if new_content == content:
            # Try simpler pattern
            pattern2 = rf'(pub async fn {handler.function_name}\()'
            # Check if there's already an attribute above
            check = re.search(rf'#\[[^\]]+\]\s*\n(pub async fn {handler.function_name}\()', content)
            if check:
                # Insert before existing attributes
                pattern3 = rf'(#\[[^\]]+\]\s*\n)(pub async fn {handler.function_name}\()'
                new_content = re.sub(pattern3, rf'{annotation}\n\1\2', content, count=1)

        return new_content

    def apply_to_all(self, handlers_dir: Path, target_file: Optional[str] = None):
        """Apply annotations to all handler files"""
        gen = OpenAPIGenerator(handlers_dir)

        # Get all handler files
        files = list(handlers_dir.glob('*_handlers.rs'))
        if target_file:
            files = [f for f in files if f.name == target_file]
            if not files:
                print(f"❌ File not found: {target_file}")
                return

        print(f"📝 Processing {len(files)} handler files...\n")

        for file_path in files:
            if file_path.name == 'mod.rs':
                continue

            print(f"🔧 Processing {file_path.name}")

            # Parse handlers from file
            handlers = gen.parse_handler_file(file_path)

            if not handlers:
                print(f"  ⚠️  No handlers found in {file_path.name}")
                continue

            # Apply annotations
            self.apply_to_file(file_path, handlers)

        # Summary
        print(f"\n{'='*60}")
        print(f"📊 Summary:")
        print(f"  ✅ Modified files: {len(self.modified_files)}")
        print(f"  ❌ Failed files: {len(self.failed_files)}")

        if self.failed_files:
            print(f"\n⚠️  Failed files:")
            for file_path, error in self.failed_files:
                print(f"  - {file_path.name}: {error}")

        if self.dry_run:
            print(f"\n🔍 This was a DRY RUN. No files were modified.")
            print(f"   Run without --dry-run to apply changes.")


def main():
    """Main entry point"""
    # Parse arguments
    dry_run = '--dry-run' in sys.argv
    create_backup = '--no-backup' not in sys.argv
    target_file = None

    for arg in sys.argv[1:]:
        if arg.startswith('--file='):
            target_file = arg.split('=')[1]

    # Get backend directory
    backend_dir = Path(__file__).parent.parent
    handlers_dir = backend_dir / 'src' / 'infrastructure' / 'web' / 'handlers'

    if not handlers_dir.exists():
        print(f"❌ Handlers directory not found: {handlers_dir}")
        return 1

    print("🚀 OpenAPI Annotation Applier")
    print(f"{'='*60}")
    print(f"Mode: {'DRY RUN' if dry_run else 'APPLY CHANGES'}")
    print(f"Backup: {'Enabled' if create_backup else 'Disabled'}")
    print(f"Target: {target_file if target_file else 'All files'}")
    print(f"{'='*60}\n")

    # Create applier
    applier = AnnotationApplier(dry_run=dry_run, create_backup=create_backup)

    # Apply annotations
    applier.apply_to_all(handlers_dir, target_file)

    print(f"\n✅ Done!")

    return 0


if __name__ == '__main__':
    exit(main())
