Pod::Spec.new do |s|
  s.name             = 'DashSharedCore'
  s.version          = '0.4.0'
  s.summary          = 'Dash Core SPV written in Rust'
  s.author           = 'Dash'
  s.description      = "C-bindings for Dash Core SPV that can be used in projects for Apple platform"
  s.homepage         = 'https://github.com/dashpay/dash-shared-core'

  s.license          = { :type => 'MIT', :file => 'LICENSE' }
  s.source           = { :git => 'https://github.com/dashpay/dash-shared-core.git', :tag => s.version.to_s }

  s.ios.deployment_target = '13.0'
  s.macos.deployment_target = '10.15'

  s.prepare_command = <<-CMD
    cd dash-spv-apple-bindings
    ./build.sh
  CMD

  s.source_files = 'dash-spv-apple-bindings/DashSharedCore/include/**/*.h'
  s.ios.vendored_frameworks = 'dash-spv-apple-bindings/DashSharedCore/framework/DashSharedCore.xcframework'
  s.osx.vendored_libraries = 'dash-spv-apple-bindings/DashSharedCore/lib/macos/**/*.a'
end
