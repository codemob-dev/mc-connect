package com.codemob;

import net.fabricmc.mappingio.MappingReader;
import net.fabricmc.mappingio.tree.MappingTreeView;
import net.fabricmc.mappingio.tree.MemoryMappingTree;
import org.jetbrains.annotations.Nullable;
import org.w3c.dom.Document;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;
import org.xml.sax.SAXException;

import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import java.io.*;
import java.net.URI;
import java.net.URL;
import java.net.URLConnection;
import java.util.HashMap;
import java.util.List;
import java.util.Objects;
import java.util.zip.GZIPInputStream;

public class YarnMappingResolver {
    private final MemoryMappingTree mappingTree = new MemoryMappingTree();
    private final HashMap<String, Integer> namespaceMapping = new HashMap<>();

    public YarnMappingResolver(String mcVersion) throws Exception {
        String build = getLatestBuild(mcVersion);
        File mappings = downloadMappings(build);
        MappingReader.read(mappings.toPath(), mappingTree);
        List<String> namespaces = mappingTree.getDstNamespaces();
        for (int i = 0; i < namespaces.size(); i++) {
            namespaceMapping.put(namespaces.get(i), i);
        }
    }

    private String getLatestBuild(String version) throws IOException, SAXException, ParserConfigurationException {
        String sURL = "https://maven.fabricmc.net/net/fabricmc/yarn/maven-metadata.xml";

        URL url = URI.create(sURL).toURL();
        URLConnection request = url.openConnection();
        request.connect();

        DocumentBuilder newDocumentBuilder = DocumentBuilderFactory.newInstance().newDocumentBuilder();
        Document parse = newDocumentBuilder.parse(request.getInputStream());
        NodeList elements = parse.getElementsByTagName("version");

        int latestVersionNumber = -1;
        String latestVersion = null;
        for (int i = 0; i < elements.getLength(); i++) {
            String build = elements.item(i).getTextContent();
            String mcVersion = build.split("\\+")[0];
            if (mcVersion.equals(version)) {
                int versionNumber = Integer.parseInt(build.split("build\\.")[1]);
                if (versionNumber > latestVersionNumber) {
                    latestVersionNumber = versionNumber;
                    latestVersion = build;
                }
            }
        }
        return latestVersion;
    }

    private File downloadMappings(String build) throws IOException {
        String downloadUrl = "https://maven.fabricmc.net/net/fabricmc/yarn/" + build + "/yarn-" + build + "-tiny.gz";
        File tmpFile = File.createTempFile("yarn-mappings", ".tiny");
        tmpFile.deleteOnExit();

        URLConnection conn = URI.create(downloadUrl).toURL().openConnection();
        try (
                InputStream in = new GZIPInputStream(conn.getInputStream());
                FileOutputStream out = new FileOutputStream(tmpFile)
        ) {
            in.transferTo(out);
        }

        return tmpFile;
    }

    public String getClassMapping(String className) {
        return mappingTree.mapClassName(
                className,
                namespaceMapping.get("named"),
                namespaceMapping.get("intermediary")
        );
    }

    public String getMethodMapping(String className, String methodName, @Nullable String desc) {
        return Objects.requireNonNull(mappingTree.getMethod(
                        className, methodName, desc, namespaceMapping.get("named")))
                .getDstName(namespaceMapping.get("intermediary"));
    }

    public String getFieldMapping(String className, String fieldName, @Nullable String desc) {
        return Objects.requireNonNull(mappingTree.getField(
                        className, fieldName, desc, namespaceMapping.get("named")))
                .getDstName(namespaceMapping.get("intermediary"));
    }

    public MemoryMappingTree getMappingTree() {
        return mappingTree;
    }
}
