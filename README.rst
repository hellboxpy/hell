hell
====

``hell init``

Sets up a new project by:

* Creating a new Python 3 virtual environment
* Installing ``hellbox`` and creating a ``Pipfile``
* Creating a minimal ``Hellfile.py`` for defining tasks

``hell run {task}``

Runs the task defined in ``Hellfile.py``. Defaults to the task named ``default``.

``hell install``

Installs all packages in ``Pipfile`` into the project's Python installation.

``hell install {package}``

Installs a package using ``pipenv`` into the project's Python installation

``hell uninstall {package}``

Uninstalls a package using ``pipenv`` from the project's Python installation

``hell inspect``

Loads the ``Hellfile.py`` manifest and displays the defined tasks:

.. code-block::

  │ » build
  │   Builds font files from source
  ╽
  ┗━ ReadFiles(globs=('source/*.ufo',))
     ┣━ GenerateOtf
     ┃  ┗━ RenameFile(prefix='16th-Century-Gothic-')
     ┃     ┗━ DummyDsig
     ┃        ┗━ Autohint
     ┃           ┗━ Write(path='build/otf')
     ┗━ GenerateTtf
        ┗━ RenameFile(prefix='16th-Century-Gothic-')
           ┗━ DummyDsig
              ┗━ Autohint
                 ┣━ GenerateWoff2
                 ┃  ┗━ Write(path='build/woff2')
                 ┗━ Write(path='build/ttf')
