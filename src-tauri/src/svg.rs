use crate::metadata::{self, AppResult, Metadata, DC};
use quick_xml::{events::Event, name::ResolveResult, NsReader};
use std::{collections::BTreeMap, fs, path::Path};
use xmp_toolkit::{ToStringOptions, XmpMeta};

const SVG: &str = "http://www.w3.org/2000/svg";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

struct Element {
    name: String,
    local: String,
    ns: String,
    start: usize,
    end: usize,
    inner_start: usize,
    inner_end: usize,
    parent: Option<usize>,
    bindings: BTreeMap<String, String>,
    empty: bool,
}

// Retain source byte ranges so writing metadata never reserializes artwork.
struct Document {
    text: String,
    elements: Vec<Element>,
}

impl Document {
    fn parse(text: String) -> AppResult<Self> {
        let mut reader = NsReader::from_str(&text);
        let mut elements: Vec<Element> = Vec::new();
        let mut stack = Vec::new();
        loop {
            let start = reader.buffer_position() as usize;
            let (namespace, event) = reader
                .read_resolved_event()
                .map_err(|e| format!("XML tidak valid: {e}"))?;
            let ns = match namespace {
                ResolveResult::Bound(value) => value.as_ref().to_owned(),
                ResolveResult::Unbound => String::new(),
                ResolveResult::Unknown(_) => {
                    return Err("Prefix namespace XML tidak dikenal.".into())
                }
            };
            match event {
                Event::Start(ref tag) | Event::Empty(ref tag) => {
                    if stack.is_empty() && !elements.is_empty() {
                        return Err("XML memiliki lebih dari satu elemen root.".into());
                    }
                    let empty = matches!(event, Event::Empty(_));
                    let mut bindings = BTreeMap::new();
                    for attr in tag.attributes() {
                        let attr = attr.map_err(|e| e.to_string())?;
                        let key = attr.key.as_ref().to_owned();
                        if key == "xmlns" || key.starts_with("xmlns:") {
                            bindings.insert(
                                key,
                                attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                    .map_err(|e| e.to_string())?
                                    .into_owned(),
                            );
                        }
                    }
                    let end = reader.buffer_position() as usize;
                    elements.push(Element {
                        name: tag.name().as_ref().to_owned(),
                        local: tag.local_name().as_ref().to_owned(),
                        ns,
                        start,
                        end,
                        inner_start: end,
                        inner_end: end,
                        parent: stack.last().copied(),
                        bindings,
                        empty,
                    });
                    if !empty {
                        stack.push(elements.len() - 1);
                    }
                }
                Event::End(_) => {
                    let index = stack.pop().ok_or("Penutup XML tanpa pembuka.")?;
                    elements[index].inner_end = start;
                    elements[index].end = reader.buffer_position() as usize;
                }
                Event::Decl(declaration) => {
                    if let Some(encoding) = declaration.encoding() {
                        let encoding = encoding.map_err(|e| e.to_string())?;
                        if !encoding.eq_ignore_ascii_case("utf-8")
                            && !encoding.eq_ignore_ascii_case("utf8")
                        {
                            return Err("SVG harus menggunakan encoding UTF-8.".into());
                        }
                    }
                }
                Event::Eof => break,
                _ => (),
            }
        }
        if elements.is_empty() || !stack.is_empty() {
            return Err("Dokumen XML tidak lengkap.".into());
        }
        Ok(Self { text, elements })
    }

    fn load(path: &Path) -> AppResult<Self> {
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
            return Err("SVG melebihi batas 64 MiB untuk penyuntingan metadata.".into());
        }
        let document = Self::parse(
            fs::read_to_string(path).map_err(|e| format!("SVG harus berupa UTF-8: {e}"))?,
        )?;
        let root = &document.elements[0];
        if root.local != "svg" || (!root.ns.is_empty() && root.ns != SVG) {
            return Err("Root dokumen bukan SVG.".into());
        }
        Ok(document)
    }

    fn root_child(&self, local: &str) -> Option<usize> {
        self.elements
            .iter()
            .position(|e| e.parent == Some(0) && e.local == local && e.ns == self.elements[0].ns)
    }

    fn packet(&self) -> Option<usize> {
        self.elements.iter().enumerate().position(|(i, e)| {
            if e.local != "RDF" || e.ns != RDF {
                return false;
            }
            let mut parent = self.elements[i].parent;
            while let Some(index) = parent {
                let node = &self.elements[index];
                if node.parent == Some(0)
                    && node.local == "metadata"
                    && node.ns == self.elements[0].ns
                {
                    return true;
                }
                parent = node.parent;
            }
            false
        })
    }

    fn xmp(&self) -> AppResult<XmpMeta> {
        let Some(index) = self.packet() else {
            return XmpMeta::new().map_err(|e| e.to_string());
        };
        let mut ancestors = Vec::new();
        let mut parent = Some(index);
        while let Some(i) = parent {
            ancestors.push(i);
            parent = self.elements[i].parent;
        }
        let mut bindings = BTreeMap::new();
        for i in ancestors.into_iter().rev() {
            bindings.extend(self.elements[i].bindings.clone());
        }
        bindings.insert("xmlns:x".into(), "adobe:ns:meta/".into());
        let attrs = bindings
            .iter()
            .map(|(key, value)| format!(" {key}=\"{}\"", quick_xml::escape::escape(value)))
            .collect::<String>();
        let node = &self.elements[index];
        format!(
            "<x:xmpmeta{attrs}>{}</x:xmpmeta>",
            &self.text[node.start..node.end]
        )
        .parse()
        .map_err(|e: xmp_toolkit::XmpError| e.to_string())
    }

    fn text_content(&self, index: usize) -> AppResult<String> {
        let node = &self.elements[index];
        let mut reader = quick_xml::Reader::from_str(&self.text[node.inner_start..node.inner_end]);
        let mut value = String::new();
        loop {
            match reader.read_event().map_err(|e| e.to_string())? {
                Event::Text(text) => value.push_str(
                    &quick_xml::escape::unescape(&text.as_ref()).map_err(|e| e.to_string())?,
                ),
                Event::CData(text) => value.push_str(&text.as_ref()),
                Event::GeneralRef(reference) => {
                    let encoded = format!("&{};", reference.as_ref());
                    value.push_str(
                        &quick_xml::escape::unescape(&encoded).map_err(|e| e.to_string())?,
                    );
                }
                Event::Eof => break,
                _ => (),
            }
        }
        Ok(value)
    }
}

pub fn read(path: &Path) -> AppResult<Metadata> {
    let doc = Document::load(path)?;
    let mut result = metadata::fields(&doc.xmp()?);
    for (tag, value) in [
        ("title", &mut result.title),
        ("desc", &mut result.description),
    ] {
        if value.is_empty() {
            if let Some(index) = doc.root_child(tag) {
                *value = doc.text_content(index)?;
            }
        }
    }
    // Some SVG exports store plain Dublin Core children in the metadata element.
    if let Some(meta) = doc.root_child("metadata") {
        for (index, node) in doc.elements.iter().enumerate() {
            if node.parent != Some(meta) || node.ns != DC {
                continue;
            }
            let text = doc.text_content(index)?;
            match node.local.as_str() {
                "title" if result.title.is_empty() => result.title = text,
                "description" if result.description.is_empty() => result.description = text,
                "subject" if result.keywords.is_empty() => {
                    result.keywords = text
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect()
                }
                _ => (),
            }
        }
    }
    Ok(result)
}

pub fn write(path: &Path, value: &Metadata) -> AppResult<()> {
    let doc = Document::load(path)?;
    let mut xmp = doc.xmp()?;
    metadata::update(&mut xmp, value)?;
    let serialized = xmp
        .to_string_with_options(ToStringOptions::default())
        .map_err(|e| e.to_string())?;
    let packet_doc = Document::parse(serialized)?;
    let rdf = packet_doc
        .elements
        .iter()
        .find(|e| e.local == "RDF" && e.ns == RDF)
        .ok_or("Paket RDF tidak tersedia.")?;
    let packet = &packet_doc.text[rdf.start..rdf.end];
    let root = &doc.elements[0];
    let prefix = root
        .name
        .rsplit_once(':')
        .map(|(prefix, _)| format!("{prefix}:"))
        .unwrap_or_default();
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut insert = String::new();
    for (tag, text) in [("title", &value.title), ("desc", &value.description)] {
        let replacement = if text.is_empty() {
            String::new()
        } else {
            format!(
                "<{prefix}{tag}>{}</{prefix}{tag}>",
                quick_xml::escape::escape(text)
            )
        };
        if let Some(i) = doc.root_child(tag) {
            let node = &doc.elements[i];
            edits.push((node.start, node.end, replacement));
        } else {
            insert.push_str(&replacement);
        }
    }
    if let Some(i) = doc.packet() {
        let node = &doc.elements[i];
        edits.push((node.start, node.end, packet.into()));
    } else if let Some(i) = doc.root_child("metadata") {
        let node = &doc.elements[i];
        if node.empty {
            edits.push((
                node.start,
                node.end,
                format!("<{prefix}metadata>{packet}</{prefix}metadata>"),
            ));
        } else {
            edits.push((node.inner_end, node.inner_end, packet.into()));
        }
    } else {
        insert.push_str(&format!("<{prefix}metadata>{packet}</{prefix}metadata>"));
    }
    for node in &doc.elements {
        if node.ns == DC
            && matches!(node.local.as_str(), "title" | "description" | "subject")
            && node
                .parent
                .map(|i| doc.elements[i].parent == Some(0) && doc.elements[i].local == "metadata")
                .unwrap_or(false)
        {
            edits.push((node.start, node.end, String::new()));
        }
    }
    if root.empty {
        edits.push((
            root.start,
            root.end,
            format!(
                "{}>{insert}</{}>",
                &doc.text[root.start..root.end - 2],
                root.name
            ),
        ));
    } else if !insert.is_empty() {
        edits.push((root.inner_start, root.inner_start, insert));
    }
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut output = doc.text;
    for (start, end, replacement) in edits {
        output.replace_range(start..end, &replacement);
    }
    Document::parse(output.clone())?;
    fs::write(path, output).map_err(|e| e.to_string())
}
