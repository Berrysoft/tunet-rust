import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:url_launcher/url_launcher_string.dart';

class AboutPage extends StatelessWidget {
  const AboutPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        children: [
          Image.asset("assets/logo.png", width: width / 3, height: width / 3),
          Card(
            child: Column(
              children: [
                const ListTile(
                  leading: Icon(Icons.info_outline_rounded),
                  title: Text('清华大学校园网客户端'),
                ),
                const ListTile(
                  leading: Icon(Icons.copyright_rounded),
                  title: Text('2023 Berrysoft'),
                ),
                const ListTile(
                  leading: FlutterLogo(),
                  title: Text('使用 Flutter 开发'),
                ),
                ListTile(
                  leading: const Icon(Icons.build_rounded),
                  title: FutureBuilder(
                    future: PackageInfo.fromPlatform(),
                    builder: (context, snap) {
                      final info = snap.data;
                      if (info == null) {
                        return const LinearProgressIndicator();
                      }
                      return Text('${info.version}:${info.buildNumber}');
                    },
                  ),
                ),
                InkWell(
                  onTap: () async {
                    const url = "https://github.com/Berrysoft/tunet-rust/";
                    await launchUrlString(url);
                  },
                  child: const ListTile(
                    leading: FaIcon(FontAwesomeIcons.github),
                    title: Text('Berrysoft/tunet-rust'),
                  ),
                ),
                InkWell(
                  onTap: () async {
                    const url = "tel:01062784859";
                    await launchUrlString(url);
                  },
                  child: const ListTile(
                    leading: Icon(Icons.dialpad_rounded),
                    title: Text('010-62784859'),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
