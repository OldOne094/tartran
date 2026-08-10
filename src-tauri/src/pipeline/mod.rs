use crate::error::{AppError, AppResult};
use crate::llm::{GeminiProvider, TranslateRequest, DEFAULT_MODEL};
use crate::models::{CreateSuggestionInput, Suggestion, TranslateChapterInput, TranslateResult};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::glossary::GlossaryManager;
use crate::storage::project_store::ProjectStore;
use crate::storage::suggestions::SuggestionsManager;
use crate::text::{is_mostly_cjk, measure, strip_arabic_diacritics};
use chrono::Utc;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

const MAX_ZH_BUDGET: usize = 350;
const MAX_EN_BUDGET: usize = 500;
const INTER_REQUEST_DELAY_MS: u64 = 1500;
const MAX_429_RETRIES: u32 = 3;
const PREVIOUS_SUMMARY_LIMIT: usize = 5;

const ARABIC_LITERARY_TRANSLATOR_PROMPT: &str = r##"# نظام الترجمة السياقية للأعمال الأدبية الصينية

أنت **مترجم أدبي محترف** متخصص في تحويل الأعمال الأدبية الصينية إلى العربية، وتمتلك خبرة تراكمية في النقل الدقيق للمعنى مع الحفاظ على الطبيعة والسلاسة.

---

## [الدور]

مهمتك الأساسية هي إنتاج **ترجمة أدبية عالية الجودة باللغة العربية الفصحى** من النصوص الصينية الأصلية. أنت لست مجرد مترجم آلي، بل مترجم يقرأ النص بعمق ويعيد إنتاجه بروح عربية سليمة. يجب أن تتعامل مع كل مقطع بإتقان تام.

---

## [الهدف النهائي]

إنتاج ترجمة:

- **طبيعية وسلسة** تُقرأ كأنها نُكبت بالعربية أصلاً.
- **وفيِة للمعنى** دون زيادة أو نقصان أو حذف أو تلخيص.
- **متسقة** من حيث المصطلحات والأسماء والأحداث والتوقيتات.
- **احترافية** خالية من الأخطاء اللغوية والإملائية والنحوية، مع ضبط الشكل الكامل.

---

## [قواعد الترجمة]

1. **الصياغة الطبيعية**:
   - أعد صياغة الجمل لتوافق التعبير العربي الفصيح، لا ترجم حرفيًا.
   - تجنب الركاكة، والحشو، والتعقيد غير الضروري.
   - استخدم بناءً عربيًا سليمًا في البنية والتركيب اللغوي.

2. **الفصحى والوضوح**:
   - التزم بالفصحى الواضحة، بعيدًا عن الركاكة أو الحشو أو التقعر.
   - لا تستخدم العامية المفرطة أو الركيكة إلا في المواضع التي يقتضيها السياق الداخلي للعمل (مثل حوار شخصيات عامية في الأصل)، وبشكل متحفظ وواضح.

3. **الترتيب الداخلي**:
   - لا تغيّر ترتيب الأفكار أو الفقرات إلا عند الحاجة اللغوية الضرورية للسلاسة.
   - احترم بنية الجمل والمشاهد وتناسق الأحداث الزمني.

4. **الدقة والوفاء**:
   - ترجم المعنى بدقة ووفاء كاملين.
   - لا تحذف أفكارًا أو تقصها أو تضيفها، إلا ما تفرضه ضرورة الصياغة العربية.

5. **الاتساق**:
   - حافظ على اتساق ترجمة الأسماء والألقاب والأماكن والمصطلحات مع ما سبق في العمل.
   - تذكّر قراراتك السابقة في الترجمة، ولا تغيّرها إلا لسبب قوي، وعندها يجب إبراز ذلك بوضوح (وليس في الترجمة النهائية نفسها).

6. **التعامل مع العبارات الصينية**:
   - عبارات مثل "她笑了" و"他点了点头" تُترجم بعبارات عربية طبيعية ("ابتسمت"، "أومأ برأسه") لا بعبارات حرفية ركيكة.
   - عبارات مثل "她的心猛地一沉" تُترجم إلى شيء مثل "غاص قلبها فجأة" أو "انقبض صدرها فجأة" بما يليق بالسياق.

7. **ضبط الشكل (التشكيل)**:
   - اضبط حركات جميع حروف الأسماء والأفعال والصفات، وأشكل الحروف المهمة.
   - راجع النص المشرك لتتأكد من صحة الضبط في مواضع التفخيم والترقيق، والهمزات، والمدود، والتنوين، والسكون، والشدّة.
   - تأكد من أن الضبط لا يغيّر المعنى ولا يوحي بمعنى خاطئ.
   - قدّم الترجمة مشكولةً تشكيلًا كاملًا، حتى لو كان التشكيل الكامل يزيد طول النص.

8. **الجودة النهائية**:
   - راجع النص قبل تسليمه، وتأكد من أنه مكتوب بأفضل صياغة ممكنة.
   - لا تعرض على القارئ أي ملاحظات أو شروح أو تحليل في النص النهائي؛ فالتسليم النهائي هو الترجمة نفسها فقط.

---

## [توجيهات الأسلوب]

**الهدف:**
إنتاج ترجمة أدبية راقية، تقدم تجربة قراءة ممتعة وقيمة، تحافظ على روح النص الأصلي وتميّزه.

**كيف تحقق ذلك؟**
- اقرأ الجملة الصينية بعمق، ثم أعد إنتاجها بالعربية كما لو كانت الأصل.
- احذف الألفاظ الركيكة والحشو غير الضروري الذي لا يخدم المعنى أو الجو.
- استخدم مفردات فصيحة ومتنوعة، وتراكيب جميلة ومرنة.
- حافظ على الإيقاع الطبيعي والسلاسة بين الجمل والفقرات.
- عند ترجمة الحوار، اجعله معبرًا وطبيعيًا كما ينطق به المتحدث.

---

## [معايير الرفض] — لماذا تفشل بعض الترجمات؟

الترجمات الآلية تفشل عادة للأسباب التالية، وكلها **مرفوضة** في عملك:

| السبب | مثال تقريبي |
|---|---|
| الترجمة الحرفية | ترجمة "她笑了" إلى "هي ابتسمت" بدل "ابتسمت" |
| الجمود والركاكة | أسلوب نثري متصلب لا يشبه العربية السليمة |
| الحشو الزائد | إضافة كلمات لا تخدم النص (مثل "تلك"، "إنه"، "لقد" بشكل مفرط) |
| ضعف التشكيل | ترك الكلمات بلا ضبط أو بضبط ناقص |
| الترتيب الخاطئ | الجمل غير المرتبة ضمن الفقرة أو تسلسلها |
| عدم الاتساق | تغيّر ترجمة اسم أو مصطلح داخل العمل |

**الخلاصة:** لا تُنتج ترجمة "مشروحة" ولا ترجمة "آلية"، بل ترجمة أدبية مكتملة، مُشكلة، وطبيعية تمامًا.

---

## [المصطلحات المتوافقة]

التزم بالمصطلحات المتوفرة في قاموسك الموثوق (المصطلحات المعتمدة) عند ترجمتها.

---

## [المصطلحات غير المقررة]

إذا قابلت مصطلحًا غير محدد بعد:

- ترجمه بما يناسب سياق الاستخدام في النص.
- اجعله متسقًا في جميع ظهوره لاحقًا.
- قدّمه ضمن اقتراحات القاموس إن كان متكررًا وهامًا.

---

## [الأسماء الشخصية]

- حافظ على اتساق ترجمة أسماء الشخصيات.
- تُنقل الأسماء الصينية في العادة نقلًا صوتيًا إلى العربية مع ضبطها، ما لم تكن متفقًا عليها في القاموس المعتمد.
- راعِ الفروق اللغوية والثقافية عند ترجمة الكنى والألقاب (مثل: 师父، 前辈، 师弟، 师兄، 姐姐، 妹妹).

---

## [صيغة الإدخال]

سيصلك النص في هذا الترتيب الثابت:

```
[PROJECT INSTRUCTIONS]
[TRANSLATION STYLE]
[APPROVED GLOSSARY]
[CHARACTER / ENTITY CONTEXT]
[PREVIOUS CONTEXT - OPTIONAL]
[CHAPTER CONTEXT]
[SOURCE TEXT]
```

التزم بهذا الترتيب، ولا تتوقع أقسامًا خارجه.

---

## [صيغة الإخراج]

الإخراج الافتراضي يجب أن يحتوي على:

```
[FINAL ARABIC TRANSLATION]
```

ثم **الترجمة فقط** دون أي شيء آخر، بما في ذلك عدم إدراج أي ملاحظات أو تحليلات أو شروح أو مقارنات.

> ملاحظة: إذا كان لديك اقتراحات مصطلحات، قُدّمها بشكل منفصل ضمن اقتراحات القاموس المعتمدة في النظام، وليس داخل الترجمة النهائية.

---

## [القاعدة الذهبية]

**لا تفكّر في الحوافز. فكّر في الترجمة.**

لا تحاول إرضاء أي نظام تصحيح أو تجميل بإضافة تعليقات أو "بسمة" إضافية داخل الترجمة النهائية. كل ما يُطلب منك هو:
1. اقرأ النص بعناية.
2. أعد إنتاج الأدب بالعربية بأفضل صياغة.
3. سلّم الترجمة فقط، دون أي شيء خارجها.
"##;

const ENGLISH_SOURCE_ARABIC_PROMPT: &str = r##"# نظام الترجمة السياقية للأعمال الأدبية الإنجليزية

أنت **مترجم أدبي محترف** متخصص في تحويل الأعمال الأدبية المكتوبة بالإنجليزية إلى العربية، وتمتلك خبرة تراكمية في النقل الدقيق للمعنى مع الحفاظ على الطبيعة والسلاسة.

---

## [الدور]

مهمتك الأساسية هي إنتاج **ترجمة أدبية عالية الجودة باللغة العربية الفصحى** من النصوص الإنجليزية الأصلية. أنت لست مجرد مترجم آلي، بل مترجم يقرأ النص بعمق ويعيد إنتاجه بروح عربية سليمة.

---

## [الهدف النهائي]

إنتاج ترجمة:

- **طبيعية وسلسة** تُقرأ كأنها كُتبت بالعربية أصلاً.
- **وفيِة للمعنى** دون زيادة أو نقصان أو حذف أو تلخيص.
- **متسقة** من حيث المصطلحات والأسماء والأحداث والتوقيتات.
- **احترافية** خالية من الأخطاء اللغوية والإملائية والنحوية.

---

## [قواعد الترجمة]

1. **الصياغة الطبيعية**: أعد صياغة الجمل لتوافق التعبير العربي الفصيح، ولا ترجم حرفيًا.
2. **الفصحى والوضوح**: التزم بالفصحى الواضحة، بعيدًا عن الركاكة أو الحشو أو التقعر.
3. **الدقة والوفاء**: لا تحذف أفكارًا أو تقصها أو تضيفها، إلا ما تفرضه ضرورة الصياغة العربية.
4. **الاتساق**: حافظ على اتساق ترجمة الأسماء والألقاب والأماكن والمصطلحات مع ما سبق في العمل.

## [الأسماء الشخصية]

- حافظ على اتساق ترجمة أسماء الشخصيات، وترجمها ترجمة صوتية طبيعية ومستقرة.
- **لا تختلق أسماءً عشوائية** ولا تعطِ شخصياتٍ أسماءً جديدة غير موجودة في النص.
- الأسماء الإنجليزية الشائعة تُنقل صوتيًا بأشهر صيغة عربية (مثل John → جون، Sarah → سارة).

---

## [صيغة الإدخال]

سيصلك النص في هذا الترتيب الثابت:

```
[PROJECT INSTRUCTIONS]
[TRANSLATION STYLE]
[APPROVED GLOSSARY]
[CHARACTER / ENTITY CONTEXT]
[PREVIOUS CONTEXT - OPTIONAL]
[CHAPTER CONTEXT]
[SOURCE TEXT]
```

## [صيغة الإخراج]

الإخراج الافتراضي يجب أن يحتوي على الترجمة فقط دون أي شيء آخر، بما في ذلك عدم إدراج أي ملاحظات أو تحليلات أو شروح.

## [القاعدة الذهبية]

سلّم الترجمة فقط، دون أي شيء خارجها.
"##;

pub struct TranslationPipeline<'a> {
    settings: AppSettingsStore,
    logger: &'a crate::logger::Logger,
}

impl<'a> TranslationPipeline<'a> {
    pub fn new(settings: AppSettingsStore, logger: &'a crate::logger::Logger) -> Self {
        TranslationPipeline { settings, logger }
    }

    fn store_for(&self, project_id: &str) -> AppResult<ProjectStore> {
        let entry = self
            .settings
            .registry()
            .into_iter()
            .find(|e| e.id == project_id)
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {project_id}")))?;
        ProjectStore::open(&PathBuf::from(&entry.path).join("novel.db"))
            .map_err(|e| AppError::Db(e))
    }

    pub fn translate_chapter(
        &self,
        project_id: &str,
        input: &TranslateChapterInput,
        limiter: &Mutex<crate::llm::rate_limiter::RateLimiter>,
        on_progress: &dyn Fn(usize, usize),
    ) -> AppResult<TranslateResult> {
        let started = Instant::now();
        let store = self.store_for(project_id)?;
        let row = store
            .get_chapter(&input.chapter_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Chapter not found".into()))?;
        if row.source_text.trim().is_empty() {
            return Err(AppError::InvalidInput("Chapter has no source text".into()));
        }

        let meta = store.get_meta().map_err(|e| AppError::Db(e))?;
        let target_lang = meta.get("targetLang").cloned().unwrap_or_else(|| "ar".into());
        let project_title = meta.get("title").cloned().unwrap_or_default();

        // The script of the source text decides which literary prompt to use and how
        // glossary terms are matched (zh for CJK, en for Latin).
        let source_is_cjk = is_mostly_cjk(&row.source_text);
        let app_settings = self.settings.get();
        let remove_tashkeel = app_settings.remove_tashkeel && target_lang != "en";
        let temperature = app_settings.temperature;

        let glossary = GlossaryManager::new(self.settings.clone(), self.logger);
        let terms = glossary.detect_terms(project_id, &row.source_text, 50, source_is_cjk)?;

        let provider = GeminiProvider::from_keyring()
            .ok_or_else(|| AppError::InvalidInput("No API key configured".into()))?;
        let model = if input.model.is_empty() {
            DEFAULT_MODEL.to_string()
        } else {
            input.model.clone()
        };

        let system_prompt =
            build_system_prompt(&target_lang, &project_title, &terms, source_is_cjk, remove_tashkeel);
        let previous_context = load_previous_context(&store, row.number, PREVIOUS_SUMMARY_LIMIT)?;
        let chunks = chunk_text(&row.source_text);
        let total_chunks = chunks.len();

        let mut full_translation = String::new();
        let mut collected_suggestions: Vec<Value> = Vec::new();
        let mut total_tokens = 0u64;

        for (i, chunk) in chunks.iter().enumerate() {
            on_progress(i, total_chunks);
            let user_prompt = build_user_prompt(chunk, i + 1, total_chunks, &previous_context);
            let resp =
                translate_with_retry(&provider, &model, &system_prompt, &user_prompt, temperature, limiter)?;
            let parsed = parse_translation_output(&resp.text);
            total_tokens += resp.usage_tokens;
            full_translation.push_str(&parsed.translation);
            if i < total_chunks - 1 {
                full_translation.push('\n');
            }
            collected_suggestions.extend(parsed.suggestions);
            if i + 1 < total_chunks {
                std::thread::sleep(std::time::Duration::from_millis(INTER_REQUEST_DELAY_MS));
            }
        }
        on_progress(total_chunks, total_chunks);

        let status = if store
            .get_chapter(&input.chapter_id)
            .map_err(|e| AppError::Db(e))?
            .map(|c| c.status == "exported" || c.status == "reviewed")
            .unwrap_or(false)
        {
            "reviewed"
        } else {
            "translated"
        };
        // When the user opted out of diacritics, strip tashkeel from the final output
        // as a deterministic safety net on top of the prompt-level instruction.
        let final_translation = if remove_tashkeel {
            strip_arabic_diacritics(&full_translation)
        } else {
            full_translation.clone()
        };
        store
            .update_chapter(
                &input.chapter_id,
                None,
                None,
                Some(&final_translation),
                Some(status),
                &Utc::now().to_rfc3339(),
            )
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;

        let summary = summarize_chapter(
            &provider,
            &model,
            &target_lang,
            &row.source_text,
            &terms,
            source_is_cjk,
            temperature,
            limiter,
        );
        match summary {
            Ok(s) if !s.trim().is_empty() => {
                store
                    .upsert_chapter_summary(
                        &input.chapter_id,
                        row.number,
                        &s.trim(),
                        &model,
                        &Utc::now().to_rfc3339(),
                    )
                    .map_err(|e| AppError::Db(e))?;
                self.logger.info(
                    "summary:complete",
                    Some(&json!({ "projectId": project_id, "chapterId": input.chapter_id, "chars": s.trim().len() })),
                );
            }
            Ok(_) => {
                self.logger.warn(
                    "summary:empty",
                    Some(&json!({ "projectId": project_id, "chapterId": input.chapter_id })),
                );
            }
            Err(e) => {
                self.logger.warn(
                    "summary:failed",
                    Some(&json!({ "projectId": project_id, "chapterId": input.chapter_id, "error": e.to_string() })),
                );
            }
        }

        let suggestions_manager = SuggestionsManager::new(self.settings.clone(), self.logger);
        let mut suggestions: Vec<Suggestion> = Vec::new();
        for s in collected_suggestions {
            let mut input = CreateSuggestionInput {
                chapter_id: input.chapter_id.clone(),
                zh: s.get("zh").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                en: s.get("en").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ar: s.get("ar").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                category: s.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                notes: s.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                context: s.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            };
            if remove_tashkeel {
                input.ar = strip_arabic_diacritics(&input.ar);
            }
            if !input.zh.is_empty() {
                if let Ok(sug) = suggestions_manager.create(project_id, input) {
                    suggestions.push(sug);
                }
            }
        }

        self.logger.info(
            "translation:complete",
            Some(&json!({
                "projectId": project_id,
                "chapterId": input.chapter_id,
                "model": model,
                "chunks": total_chunks,
                "suggestions": suggestions.len(),
                "tokens": total_tokens,
                "durationMs": started.elapsed().as_millis()
            })),
        );

        Ok(TranslateResult {
            chapter_id: input.chapter_id.clone(),
            translation: final_translation,
            suggestions,
            model,
            duration_ms: started.elapsed().as_millis() as u64,
            tokens_used: total_tokens,
            chunk_count: total_chunks,
        })
    }
}

/// Translate a single chunk, honoring the shared rate limiter and retrying on 429
/// with a small backoff so free-tier limits are respected.
fn translate_with_retry(
    provider: &GeminiProvider,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f64,
    limiter: &Mutex<crate::llm::rate_limiter::RateLimiter>,
) -> AppResult<crate::llm::TranslateResponse> {
    let mut attempt = 0u32;
    loop {
        {
            let guard = limiter.lock().unwrap();
            guard.acquire();
        }
        let req = TranslateRequest {
            model,
            system_prompt,
            user_prompt,
            temperature,
        };
        match provider.translate(&req) {
            Ok(resp) => return Ok(resp),
            Err(crate::llm::LlmError::RateLimited(msg)) => {
                if attempt >= MAX_429_RETRIES {
                    return Err(AppError::Llm(format!(
                        "Rate limit exceeded after retries: {msg}"
                    )));
                }
                let backoff_ms = 5000u64 * (2u64.pow(attempt));
                std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                attempt += 1;
            }
            Err(e) => return Err(AppError::Llm(e.to_string())),
        }
    }
}

struct ParsedTranslation {
    translation: String,
    suggestions: Vec<Value>,
}

fn parse_translation_output(text: &str) -> ParsedTranslation {
    let trimmed = text.trim();
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if let Some(t) = v.get("translation").and_then(|x| x.as_str()) {
            let suggestions = v
                .get("suggestions")
                .and_then(|x| x.as_array())
                .cloned()
                .unwrap_or_default();
            return ParsedTranslation {
                translation: t.to_string(),
                suggestions,
            };
        }
        if let Some(t) = v.get("translation").and_then(|x| x.as_array()) {
            let joined: Vec<String> = t.iter().filter_map(|p| p.as_str().map(String::from)).collect();
            return ParsedTranslation {
                translation: joined.join("\n"),
                suggestions: Vec::new(),
            };
        }
    }
    // Fallback: raw text is the translation.
    ParsedTranslation {
        translation: trimmed.to_string(),
        suggestions: Vec::new(),
    }
}

fn build_system_prompt(
    target_lang: &str,
    project_title: &str,
    terms: &[crate::models::GlossaryEntry],
    source_is_cjk: bool,
    remove_tashkeel: bool,
) -> String {
    let lang_name = if target_lang == "en" { "English" } else { "Arabic" };
    let mut prompt = String::new();

    if target_lang == "ar" {
        prompt.push_str(&format!("Translate into {lang_name}.\n\n"));
        if source_is_cjk {
            prompt.push_str(ARABIC_LITERARY_TRANSLATOR_PROMPT);
        } else {
            prompt.push_str(ENGLISH_SOURCE_ARABIC_PROMPT);
        }
        prompt.push('\n');
        if remove_tashkeel {
            prompt.push_str(
                "\nImportant: Output the Arabic translation WITHOUT diacritics (any vocalization/tashkeel). Plain unvocalized Arabic text only.\n",
            );
        }
    } else {
        prompt.push_str(&format!(
            "You are a professional translator of {}. Translate into {lang_name}.\n\
             Rules:\n\
             - Keep the tone and style of a web serial.\n\
             - Preserve paragraph breaks.\n\
             - Be faithful to the original meaning; do not add, omit, summarize, or explain.\n\
             - Use the provided glossary terms consistently.\n",
            if source_is_cjk { "Chinese web novels" } else { "novels" }
        ));
    }

    if !project_title.is_empty() {
        prompt.push_str(&format!("\nProject: {project_title}\n"));
    }

    if !terms.is_empty() {
        prompt.push_str("\nApproved Glossary (use these exact translations, do not retranslate):\n");
        for t in terms {
            let mut line = format!("- {} →", t.zh);
            if !t.en.is_empty() {
                line.push_str(&format!(" (en: {})", t.en));
            }
            if !t.ar.is_empty() {
                line.push_str(&format!(" (ar: {})", t.ar));
            }
            if !t.category.is_empty() {
                line.push_str(&format!(" [{}]", t.category));
            }
            if t.locked {
                line.push_str(" [LOCKED]");
            }
            prompt.push_str(&line);
            prompt.push('\n');
        }
    }

    prompt.push_str(
        "\n# [OUTPUT FORMAT - REQUIRED BY SYSTEM]\n\
         Respond ONLY with a single JSON object — no markdown fences, no text outside it:\n\
         {\"translation\": \"<the complete translated text>\", \"suggestions\": [...]}\n\
         - \"translation\": the final translation ONLY (no commentary, no headers, no notes, no review).\n\
         - \"suggestions\": proposed glossary entries, only for important recurring proper nouns / universe terms. Max 10.\n\
         - Each suggestion: {\"zh\": \"...\", \"en\": \"...\", \"ar\": \"...\", \"category\": \"character|place|item|technique|other\", \"context\": \"short context\"}\n",
    );

    prompt
}

fn build_user_prompt(chunk: &str, part: usize, total: usize, previous_context: &str) -> String {
    let mut out = String::new();
    if !previous_context.is_empty() {
        out.push_str("[PREVIOUS CONTEXT]\n");
        out.push_str(previous_context);
        out.push_str("\n\n");
    }
    out.push_str("[CHAPTER CONTEXT]\n");
    if total > 1 {
        out.push_str(&format!("Part {part}/{total}.\n"));
    }
    out.push_str("\n[SOURCE TEXT]\n");
    out.push_str(chunk);
    out
}

/// Load summaries of the most recent translated chapters before `before_number`,
/// formatted for injection under [PREVIOUS CONTEXT].
fn load_previous_context(store: &ProjectStore, before_number: i64, limit: usize) -> AppResult<String> {
    let rows = store
        .list_summaries_before(before_number, limit)
        .map_err(|e| AppError::Db(e))?;
    if rows.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::new();
    for r in &rows {
        out.push_str(&format!("## Chapter {}\n{}\n\n", r.chapter_number, r.summary));
    }
    Ok(out.trim_end().to_string())
}

/// Generate a concise summary of a translated chapter using the same Gemini model.
fn summarize_chapter(
    provider: &GeminiProvider,
    model: &str,
    target_lang: &str,
    source_text: &str,
    terms: &[crate::models::GlossaryEntry],
    source_is_cjk: bool,
    temperature: f64,
    limiter: &Mutex<crate::llm::rate_limiter::RateLimiter>,
) -> AppResult<String> {
    let system_prompt = build_summary_prompt(target_lang, terms, source_is_cjk);
    let user_prompt = format!("[SOURCE TEXT]\n\n{source_text}");
    let resp = translate_with_retry(provider, model, &system_prompt, &user_prompt, temperature, limiter)?;
    Ok(resp.text.trim().to_string())
}

fn build_summary_prompt(
    target_lang: &str,
    terms: &[crate::models::GlossaryEntry],
    source_is_cjk: bool,
) -> String {
    let lang_name = if target_lang == "en" { "English" } else { "Arabic" };
    let source_name = if source_is_cjk { "Chinese" } else { "English" };
    let mut prompt = format!(
        "You are an editor for {source_name} web novels. Write a concise summary in {lang_name} of the chapter below.\n\
         The summary will be used as memory context when translating later chapters, so it must capture:\n\
         - Key plot events and their order.\n\
         - Characters who appeared, using the approved glossary translations.\n\
         - Important locations, items, and techniques.\n\
         - Any unresolved plot threads or foreshadowing.\n\
         Keep it under 200 words, written naturally in {lang_name}.\n\
         Respond with the summary text only — no JSON, no headers, no translation of the source.\n"
    );
    if !terms.is_empty() {
        prompt.push_str("\nApproved Glossary (use these translations for names and terms):\n");
        for t in terms {
            let mut line = format!("- {} →", if source_is_cjk { &t.zh } else { &t.en });
            if !t.en.is_empty() {
                line.push_str(&format!(" (en: {})", t.en));
            }
            if !t.ar.is_empty() {
                line.push_str(&format!(" (ar: {})", t.ar));
            }
            prompt.push_str(&line);
            prompt.push('\n');
        }
    }
    prompt
}

/// Split chapter text into chunks sized for the source script:
/// Chinese ~350 chars, Latin ~500 words. Splits at paragraph boundaries
/// first, then at sentence boundaries, never mid-sentence when avoidable.
fn chunk_text(text: &str) -> Vec<String> {
    let budget = if is_mostly_cjk(text) { MAX_ZH_BUDGET } else { MAX_EN_BUDGET };
    let paragraphs: Vec<&str> = text.lines().collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for p in paragraphs {
        if p.trim().is_empty() {
            continue;
        }
        let p_size = measure(p);
        if !current.is_empty() && measure(&current) + p_size + 1 > budget {
            chunks.push(std::mem::take(&mut current));
        }
        if p_size + 1 > budget {
            // Oversized paragraph: split at sentence boundaries.
            for sentence in split_paragraph_sentences(p, budget) {
                if !current.is_empty() && measure(&current) + measure(&sentence) + 1 > budget {
                    chunks.push(std::mem::take(&mut current));
                }
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(&sentence);
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(p);
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        chunks.push(text.to_string());
    }
    chunks
}

/// Split a single paragraph into sentence-sized fragments, each under the budget.
fn split_paragraph_sentences(p: &str, budget: usize) -> Vec<String> {
    let mut fragments: Vec<String> = Vec::new();
    let mut buf = String::new();
    for c in p.chars() {
        buf.push(c);
        if matches!(c, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';' | '…') {
            fragments.push(std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        fragments.push(buf);
    }
    if fragments.is_empty() {
        return vec![p.to_string()];
    }

    // Re-group fragments so each result stays under budget.
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for f in fragments {
        if !cur.is_empty() && measure(&cur) + measure(&f) > budget {
            out.push(std::mem::take(&mut cur));
        }
        cur.push_str(&f);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_output() {
        let out = r#"{"translation":"الترجمة","suggestions":[{"zh":"剑客","ar":"سياف","category":"character"}]}"#;
        let parsed = parse_translation_output(out);
        assert_eq!(parsed.translation, "الترجمة");
        assert_eq!(parsed.suggestions.len(), 1);
    }

    #[test]
    fn falls_back_to_plain_text() {
        let parsed = parse_translation_output("نص عادي فقط");
        assert_eq!(parsed.translation, "نص عادي فقط");
        assert!(parsed.suggestions.is_empty());
    }

    #[test]
    fn user_prompt_injects_previous_context_and_sections() {
        let ctx = "## Chapter 1\nأحداث الفصل الأول.";
        let prompt = build_user_prompt("النص الحالي", 1, 3, ctx);
        assert!(prompt.contains("[PREVIOUS CONTEXT]"));
        assert!(prompt.contains("أحداث الفصل الأول."));
        assert!(prompt.contains("[CHAPTER CONTEXT]"));
        assert!(prompt.contains("[SOURCE TEXT]"));
        assert!(prompt.contains("Part 1/3."));
        assert!(prompt.ends_with("النص الحالي"));

        let no_ctx = build_user_prompt("نص وحيد", 1, 1, "");
        assert!(!no_ctx.contains("[PREVIOUS CONTEXT]"));
        assert!(!no_ctx.contains("Part"));
    }

    #[test]
    fn summary_prompt_uses_glossary() {
        let terms = vec![crate::models::GlossaryEntry {
            id: "1".into(),
            zh: "剑客".into(),
            en: "swordsman".into(),
            ar: "سياف".into(),
            category: "character".into(),
            notes: String::new(),
            aliases: vec![],
            locked: false,
            source: "manual".into(),
            created_at: String::new(),
            updated_at: String::new(),
        }];
        let prompt = build_summary_prompt("ar", &terms, true);
        assert!(prompt.contains("سياف"));
        assert!(prompt.contains("Arabic"));
    }

    #[test]
    fn chunks_at_paragraph_boundaries() {
        let mut text = String::new();
        for i in 0..200 {
            text.push_str(&format!("这是第{i}段的长句子内容，用来填充足够长的字符以便测试分块逻辑。\n"));
        }
        let chunks = chunk_text(&text);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|c| measure(c) <= MAX_ZH_BUDGET + 30));
    }

    #[test]
    fn cjk_chunks_stay_under_budget() {
        let mut text = String::new();
        for _ in 0..100 {
            text.push_str("他睁开眼，看到一片苍茫的大地。远处有一座巍峨的山峰，云雾缭绕。\n");
        }
        let chunks = chunk_text(&text);
        assert!(chunks.len() >= 2);
        for c in &chunks {
            assert!(measure(c) <= MAX_ZH_BUDGET + 30);
        }
        // Reassembly round-trip: join chunks with newline and compare content.
        let joined = chunks.join("\n");
        assert!(joined.contains("苍茫的大地"));
    }

    #[test]
    fn latin_chunks_stay_under_word_budget() {
        let text = "The sword qi tore through the sky. He stood tall, his eyes burning with resolve. ".repeat(120);
        let chunks = chunk_text(&text);
        assert!(chunks.len() >= 2);
        for c in &chunks {
            assert!(measure(c) <= MAX_EN_BUDGET + 30);
        }
    }

    #[test]
    fn empty_text_produces_one_chunk() {
        let chunks = chunk_text("");
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn system_prompt_includes_glossary() {
        let terms = vec![crate::models::GlossaryEntry {
            id: "1".into(),
            zh: "剑客".into(),
            en: "swordsman".into(),
            ar: "سياف".into(),
            category: "character".into(),
            notes: String::new(),
            aliases: vec![],
            locked: false,
            source: "manual".into(),
            created_at: String::new(),
            updated_at: String::new(),
        }];
        let prompt = build_system_prompt("ar", "Test", &terms, true, false);
        assert!(prompt.contains("剑客"));
        assert!(prompt.contains("سياف"));
        assert!(prompt.contains("Arabic"));
    }

    #[test]
    fn system_prompt_uses_english_source_prompt_for_latin_text() {
        let terms: Vec<crate::models::GlossaryEntry> = vec![];
        let prompt = build_system_prompt("ar", "", &terms, false, false);
        assert!(prompt.contains("للأعمال الأدبية الإنجليزية"));
        assert!(!prompt.contains("للأعمال الأدبية الصينية"));
    }

    #[test]
    fn system_prompt_requests_plain_arabic_when_tashkeel_removed() {
        let terms: Vec<crate::models::GlossaryEntry> = vec![];
        let prompt = build_system_prompt("ar", "", &terms, true, true);
        assert!(prompt.contains("WITHOUT diacritics"));
        let cjk_prompt = build_system_prompt("ar", "", &terms, true, false);
        assert!(!cjk_prompt.contains("WITHOUT diacritics"));
    }
}
