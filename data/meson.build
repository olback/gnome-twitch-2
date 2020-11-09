# Icons
install_data('images/net.olback.GnomeTwitch2.svg',
    install_dir: join_paths(get_option('datadir'), 'icons', 'hicolor', 'scalable', 'apps'),
)

# Desktop file
install_data('net.olback.GnomeTwitch2.desktop',
    install_dir: join_paths(get_option('datadir'), 'applications')
)

# Appdata
# install_data('net.olback.GnomeTwitch2.appdata.xml',
#     install_dir: join_paths(get_option('datadir'), 'metainfo')
# )

# G Schema
install_data('net.olback.GnomeTwitch2.gschema.xml',
    install_dir: join_paths(get_option('datadir'), 'glib-2.0', 'schemas')
)

# Compile schemas
meson.add_install_script('meson_post_install.py')

